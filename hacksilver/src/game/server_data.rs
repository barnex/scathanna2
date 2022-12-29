use super::internal::*;
use ServerMsg::*;

/// A World with automatic diffing
pub struct ServerData {
	pub world: DiffWorld,
	diffs: Diffs,
}

/// Seconds to show HUD announcements like "You fragged Foo".
const ANNOUNCE_TTL: f32 = 5.0;
const ANN_VOLUME: f32 = 1.0;

impl ServerData {
	pub fn new(world: World) -> Self {
		Self {
			world: DiffWorld::new(world),
			diffs: default(),
		}
	}

	//-------------------------------------------------------------------------------- player

	/// Player by entitiy ID.
	pub fn player(&self, id: ID) -> Option<&Player> {
		self.world.entities.players.get(id)
	}

	/// List all player IDs (does not borrow).
	pub fn players(&self) -> impl Iterator<Item = ID> {
		self.world.entities.players.copied_ids()
	}

	/// List all currently spawned player IDs (does not borrow).
	pub fn spawned_player_ids(&self) -> impl Iterator<Item = ID> {
		self.world
			.entities
			.players
			.iter()
			.filter(|(_, p)| p.spawned)
			.map(|(id, _)| id)
			.collect::<SmallVec<[_; 16]>>()
			.into_iter()
	}

	/// Player name.
	pub fn player_name(&self, id: ID) -> Option<&str> {
		self.world.entities.players.get(id).map(|p| p.name.as_str())
	}

	/// Player name, or "???"
	pub fn must_name(&self, id: ID) -> &str {
		self.player_name(id).unwrap_or("???")
	}

	/// Find player by name.
	pub fn player_by_name(&self, player_name: &str) -> Option<ID> {
		self.players().find(|&id| self.player_name(id).map(|name| name.eq_ignore_ascii_case(player_name)).unwrap_or(false))
	}

	/// Despawn player.
	pub fn despawn(&mut self, victim: ID) -> Option<()> {
		trace!("despawn {victim}");
		self.apply_to_player_partial(victim, |p| p.spawned = false)
	}

	pub fn move_player_if_spawned(&mut self, id: ID, frame: Frame) {
		self.world.move_player_if_spawned(&mut self.diffs, id, frame);
	}

	pub fn drop_player(&mut self, id: ID) {
		self.world.drop_player(&mut self.diffs, id)
	}

	/// Apply any change to a player.
	pub fn apply_to_player_partial<F: Fn(&mut Player)>(&mut self, id: ID, f: F) -> Option<()> {
		self.world.apply_to_player_partial(&mut self.diffs, id, f)
	}

	/// Apply any change to a player.
	pub fn apply_to_player_full<F: Fn(&mut Player)>(&mut self, id: ID, f: F) -> Option<()> {
		self.world.force_apply_to_full(&mut self.diffs, id, f)
	}

	pub fn join_new_player(&mut self, spawn_point: &SpawnPoint, join_msg: JoinRequest) -> (ID, MapSwitch) {
		self.world.join_new_player(&mut self.diffs, &spawn_point, join_msg)
	}

	//-------------------------------------------------------------------------------- effects

	/// Spawn an effect (for all players).
	pub fn add_effect(&mut self, effect: Effect) {
		self.diffs.push(AddEffect(effect).to_all())
	}

	pub fn sound_announce(&mut self, to: Addressee, clip_name: &'static str) {
		self.diffs.push(
			PlaySound(SoundEffect {
				clip_name: clip_name.into(),
				volume: ANN_VOLUME,
				spatial: None,
			})
			.to(to),
		)
	}

	//-------------------------------------------------------------------------------- push messages

	/// Announce message to a player's HUD.
	/// E.g. "You fragged Foo"
	pub fn hud_announce<S: ToOwned<Owned = String>>(&mut self, to: Addressee, msg: S) {
		self.push_no_apply(
			UpdateHUD(HUDUpdate {
				pos: HUDPos::TopCenter,
				text: msg.to_owned(),
				ttl_sec: ANNOUNCE_TTL,
			})
			.to(to),
		);
	}

	/// Announce message to a player's HUD, line2.
	pub fn hud_announce2<S: ToOwned<Owned = String>>(&mut self, to: Addressee, msg: S) {
		self.push_no_apply(
			UpdateHUD(HUDUpdate {
				pos: HUDPos::TopCenter2,
				text: msg.to_owned(),
				ttl_sec: ANNOUNCE_TTL,
			})
			.to(to),
		);
	}

	pub fn hud_announce_all<S: ToOwned<Owned = String>>(&mut self, msg: S) {
		self.push_no_apply(
			UpdateHUD(HUDUpdate {
				pos: HUDPos::TopCenter,
				text: msg.to_owned(),
				ttl_sec: ANNOUNCE_TTL,
			})
			.to_all(),
		);
	}

	/// Log a message to all players.
	pub fn log<S: ToOwned<Owned = String>>(&mut self, msg: S) {
		let msg = msg.to_owned();
		info!("{}", &msg);
		self.push_no_apply(Log(msg).to_all());
	}

	/// Push a message to diffs without applying to the world.
	/// (e.g. for effects etc which only need to be visible client-side).
	/// TODO: remove, replace by add_effect etc.
	pub fn push_no_apply(&mut self, msg: Envelope<ServerMsg>) {
		self.diffs.push(msg)
	}

	pub fn take_diffs(&mut self) -> Diffs {
		mem::take(&mut self.diffs)
	}

	//-------------------------------------------------------------------------------- map

	pub fn switch_map(&mut self, map: Map) {
		self.world.switch_map(&mut self.diffs, map)
	}
}
