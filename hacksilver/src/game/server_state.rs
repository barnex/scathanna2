use super::internal::*;
use Addressee::*;
use ServerMsg::*;

/// A game server's mutable state and business logic.
///
/// Owned and controlled by a NetServer, who adds an RPC layer on top.
pub struct ServerState {
	logic: GameLogic, // -> systems

	data: ServerData,
}

impl ServerState {
	pub fn new(opts: ServerOpts) -> Result<Self> {
		let assets = AssetsDir::find()?;

		let logic = GameLogic::new(assets.clone(), opts)?;

		let map = Map::load(&assets, logic.curr_map_name())?;
		let world = World::new(map, default());
		let data = ServerData::new(world);

		Ok(Self { logic, data })
	}

	/// Add a new player to the game and return their unique ID.
	pub fn join_new_player(&mut self, join_msg: JoinRequest) -> (ID, MapSwitch) {
		let (player_id, map_switch) = self.logic.join_new_player(&mut self.data, join_msg);
		self.log(format!("{} joined", self.must_name(player_id)));
		//self.hud_message(player_id, format!("Welcome to {}.", self.map_name()));
		self.data.hud_announce(Just(player_id), self.logic.curr_map_name().to_owned());
		self.push_no_apply(PlaySound(SoundEffect::raw("ann_begin")).to_just(player_id));
		(player_id, map_switch)
	}

	//-------------------------------------------------------------------------------- handlers

	pub fn handle_tick(&mut self, dt: f32) -> Diffs {
		self.logic.tick(&mut self.data, dt);
		self.data.take_diffs()
	}

	/// Respond to message sent by a player.
	pub fn handle_client_msg(&mut self, player_id: ID, msg: ClientMsg) {
		// check that the player has not been disconnected in a network race.
		// after this check, all downstream methods may safely use `self.player(id)`,
		// as we will never remove a player while handling client messages.
		if !self.data.world.entities.players.contains(player_id) {
			return;
		}

		use ClientMsg::*;
		match msg {
			MovePlayerIfSpawned(frame) => self.handle_move_player_if_spawned(player_id, frame),
			ReadyToSpawn => self.handle_ready_to_respawn(player_id),
			AddEffect(effect) => self.handle_add_effect(player_id, effect),
			PlaySound(sound) => self.handle_play_sound(player_id, sound),
			HitPlayer(victim_id) => self.handle_hit_player(player_id, victim_id),
			Command(cmd) => self.handle_command(player_id, cmd),
		};
	}

	fn handle_move_player_if_spawned(&mut self, player_id: ID, frame: Frame) {
		self.data.move_player_if_spawned(player_id, frame);
	}

	pub fn handle_hit_player(&mut self, player_id: ID, victim_id: ID) {
		self.logic.handle_hit_player(&mut self.data, player_id, victim_id);
	}

	pub fn handle_ready_to_respawn(&mut self, player_id: ID) {
		self.logic.handle_ready_to_respawn(&mut self.data, player_id)
	}

	// Handle a client's AddEffect message: just broadcast to other clients.
	// There is little point in adding visual effects to the server's world.
	pub fn handle_add_effect(&mut self, player_id: ID, effect: Effect) {
		self.push_no_apply(AddEffect(effect).to_not(player_id))
	}

	// Handle a client's PlaySound message: just broadcast to other clients.
	pub fn handle_play_sound(&mut self, player_id: ID, sound: SoundEffect) {
		self.push_no_apply(PlaySound(sound).to_not(player_id))
	}

	pub fn handle_drop_player(&mut self, client_id: ID) {
		self.log(format!("{} left", &self.must_name(client_id)));
		self.data.drop_player(client_id);
		self.logic.drop_player(client_id);
	}

	// --------------------------------------------------------------------------------

	fn broadcast_sound_at(&mut self, clip_name: &'static str, location: vec3, volume: f32) {
		self.broadcast_sound(SoundEffect::spatial(clip_name, location, volume))
	}

	fn broadcast_sound(&mut self, sound: SoundEffect) {
		self.push_no_apply(PlaySound(sound).to_all())
	}

	/// Push a message to diffs without applying to the world.
	/// (e.g. for effects etc which only need to be visible client-side).
	fn push_no_apply(&mut self, msg: Envelope<ServerMsg>) {
		self.data.push_no_apply(msg)
	}

	// Send a message to shown in the logs of all players.
	// E.g. "A killed B".
	pub fn log(&mut self, msg: String) {
		self.data.log(msg);
	}

	pub fn map_name(&self) -> &str {
		self.logic.curr_map_name()
	}

	/// Player name or "???".
	pub fn must_name(&self, player_id: ID) -> &str {
		self.data.must_name(player_id)
	}

	// -------------------------------------------------------------------------------- text commands

	fn handle_command(&mut self, client_id: ID, cmd: String) {
		info!("command from {client_id} ({}): '{cmd}'", self.must_name(client_id));
		match self.handle_command_with_result(client_id, cmd) {
			Ok(()) => info!("command ok"),
			Err(e) => {
				info!("command error: {e}");
				self.push_no_apply(Log(format!("error: {}", e)).to_just(client_id));
			}
		}
	}

	fn handle_command_with_result(&mut self, client_id: ID, cmd: String) -> Result<()> {
		Ok(match &cmd.split_ascii_whitespace().collect::<Vec<_>>()[..] {
			["say", ..] => self.handle_say_cmd(client_id, cmd["say".len()..].trim_start()),
			["switch"] => self.logic.switch_next_map(&mut self.data),
			["switch", map_name] => self.logic.switch_map_cmd(&mut self.data, map_name)?,
			["kill", victim_name] => self.handle_kill_cmd(victim_name)?,
			_ => return Err(anyhow!("unknown command: {}", cmd)),
		})
	}

	fn handle_kill_cmd(&mut self, victim_name: &str) -> Result<()> {
		let victim_id = self.data.player_by_name(victim_name).ok_or(anyhow!("no such player"))?;
		self.data.despawn(victim_id).ok_or(anyhow!("Failed (maybe monad bailed out)"))
	}

	fn handle_say_cmd(&mut self, player_id: ID, msg: &str) {
		let msg = format!("{}: {}", self.must_name(player_id), msg);
		self.push_no_apply(Log(msg).to_all());
	}
}
