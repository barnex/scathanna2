use super::internal::*;

/// ClientState stores a client's local copy of the World,
/// and provides the world-mutating methods that are allowed on the client side.
///
/// World mutations that are not allowed on the client side need to be requested from the server
/// (see ServerState).
pub struct ClientState {
	assets: AssetsDir,
	pub local_player_id: ID,
	pub world: World,
	pub hud: HUD,
	pub pending_diffs: Vec<ClientMsg>,
}

impl ClientState {
	pub fn new(ctx: &GameCtx, player_id: ID, world: World) -> Self {
		Self {
			assets: ctx.assets.clone(),
			local_player_id: player_id,
			world,
			hud: HUD::new(&ctx.graphics),
			pending_diffs: default(),
		}
	}

	// __________________________________________________________ remote control

	/// Apply a diff to the game state.
	pub fn apply_server_msg(&mut self, ctx: &GameCtx, msg: ServerMsg) {
		use ServerMsg::*;
		match msg {
			AddPlayer(player) => self.handle_add_player(player),
			MovePlayer(player_id, frame) => self.handle_move_player(player_id, frame),
			UpdatePlayerPartial(player) => self.handle_update_player_partial(player),
			UpdatePlayerFull(player) => self.handle_update_player_full(player),
			ForceMovePlayer(position) => self.handle_force_move_player(position),
			//UpdateEntity(entity) => self.handle_update_entity(entity),
			//RemoveEntity(entity_id) => self.handle_remove_entity(entity_id),
			DropPlayer(player_id) => self.handle_drop_player(player_id),
			AddEffect(effect) => self.handle_add_effect(effect),
			PlaySound(sound_effect) => self.play_sound(ctx, &sound_effect),
			//RequestRespawn(spawn_point) => self.handle_request_respawn(spawn_point),
			UpdateHUD(update) => self.handle_update_hud(update),
			SwitchMap(_) => panic!("TODO: SwitchMap currently handled by Client"), //,self.handle_switch_map(map_switch),
			Log(msg) => LOG.write(msg),
		}
	}

	//fn handle_switch_map(&mut self, map_switch: MapSwitch) {
	//  how to invalidate zonegraph??
	//	let new_map = Map::load(&self.assets, &map_switch.map_name).expect("TODO: handle map load error: {e:#}");
	//	self.world.map = new_map;
	//	self.world.entities = Entities::default().with(|e| e.players = self.world.entities.players.clone());
	//}

	fn handle_add_player(&mut self, player: Player) {
		self.world.entities.players.insert(player.id, player);
	}

	fn handle_move_player(&mut self, player_id: ID, frame: Frame) {
		if let Some(p) = self.world.entities.players.get_mut(player_id) {
			p.skeleton.set_frame(frame)
		} else {
			eprintln!("client_state: handle_move_player: player {} does not exist", player_id);
		}
	}

	// Update part of player state controlled by server: everything except frame.
	// Server sends frame anyway (for simplicity), but this is ignored here.
	// TODO: something like bevy_reflect would allow more fine-grained updates.
	fn handle_update_player_partial(&mut self, new: Player) {
		if let Some(old) = self.world.entities.players.get_mut(new.id) {
			debug_assert!(old.spawned);
			let mut new = new;
			new.local = old.local.clone();
			new.skeleton.set_frame(old.skeleton.frame());
			*old = new;
		}
	}

	// Update the entire player (including frame).
	// Server will only ask this when de-spawned.
	fn handle_update_player_full(&mut self, new: Player) {
		if let Some(player_mut) = self.world.entities.players.get_mut(new.id) {
			debug_assert!(!player_mut.spawned);
			*player_mut = new
		}
	}

	fn handle_force_move_player(&mut self, position: vec3) {
		self.local_player_mut().skeleton.position = position;
	}

	//fn handle_update_entity(&mut self, entity: Entity) {
	//	self.world.entities.insert(entity.id(), entity);
	//}

	//fn handle_remove_entity(&mut self, entity_id: EID) {
	//	self.world.entities.remove(&entity_id);
	//}

	fn handle_drop_player(&mut self, player_id: ID) {
		LOG.write("dropping player {player_id}");
		self.world.entities.players.remove(player_id);
	}

	// fn handle_request_respawn(&mut self, spawn_point: SpawnPoint) {
	// 	self.local_player_mut().next_spawn_point = spawn_point.position();
	// 	self.local_player_mut().skeleton.velocity = vec3::ZERO;
	// 	self.local_player_mut().skeleton.orientation.pitch = 0.0;
	// }

	fn handle_add_effect(&mut self, effect: Effect) {
		self.world.entities.effects.push(effect);
	}

	fn handle_update_hud(&mut self, upd: HUDUpdate) {
		trace!("handle_update_hud {:?}", &upd);
		self.hud.apply(upd);
	}

	// __________________________________________________________ sound

	fn play_sound(&self, ctx: &GameCtx, sound: &SoundEffect) {
		match &sound.spatial {
			None => self.play_sound_raw(ctx, &sound.clip_name, sound.volume),
			Some(spatial) => self.play_sound_spatial(ctx, &sound.clip_name, sound.volume, &spatial),
		}
	}

	fn play_sound_raw(&self, ctx: &GameCtx, clip_name: &str, volume: f32) {
		ctx.sound_pack.play_raw_volume(clip_name, volume)
	}

	fn play_sound_spatial(&self, ctx: &GameCtx, clip_name: &str, volume: f32, spatial: &Spatial) {
		// Sounds closer than this distance do not become any louder.
		// Otherwise very nearby sounds could become infinitely loud.

		const UNIT_DIST: f32 = 40.0;

		let player = self.local_player();
		let ear_pos = self.local_player().camera().position;
		let sound_pos = spatial.location;
		if (ear_pos - sound_pos).len() < 8.0 {
			// spatial audio does not work / is pointless when sound location is at or very near player location
			self.play_sound_raw(ctx, clip_name, volume.clamp(0.0, 1.0))
		} else {
			let azimuth = azimuth(&player.skeleton.frame(), sound_pos);
			let distance2 = (ear_pos - sound_pos).len2();
			let falloff_volume = (volume * (UNIT_DIST * UNIT_DIST) / distance2).clamp(0.0, 1.0);
			// muffle sound when obstructed by a wall
			let obstructed_volume = if self.is_obstructed(ear_pos, sound_pos) { 0.3 * falloff_volume } else { falloff_volume };
			ctx.sound_pack.play_spatial(clip_name, azimuth, obstructed_volume)
		}
	}

	// does a wall obstruct the line of sight between two positions?
	fn is_obstructed(&self, pos1: vec3, pos2: vec3) -> bool {
		let dir = (pos2 - pos1).normalized();
		let len = (pos2 - pos1).len();
		let ray = Ray64::new(pos1.into(), dir.into());
		let t = self.world.map.intersect_t(&ray).unwrap_or(f64::INFINITY) as f32;
		t < len
	}

	// __________________________________________________________ local control

	#[must_use]
	pub fn tick(&mut self, ctx: &GameCtx, inputs: &Inputs) -> ClientMsgs {
		let dt = inputs.dt();
		self.control_player(inputs, dt);
		self.extrapolate_other_players(dt);
		self.animate_footsteps(ctx, dt);
		self.tick_effects(dt);
		self.hud.tick(dt);

		let diff = mem::take(&mut self.pending_diffs);
		self.apply_self_msgs(ctx, &diff);
		diff
	}

	/// Apply a message by the local client, without round-tripping to the server.
	/// This only applies:
	///
	///   * updates to the local player, so that position/orientation don't lag by one round-trip-time.
	///   * visual effects, because these don't otherwise interact with the game state.
	///
	/// Other messages are not applied locally, but go to the server
	/// and eventually mutate the local GameState via `apply_server_msg`.
	fn apply_self_msgs(&mut self, ctx: &GameCtx, msgs: &ClientMsgs) {
		use ClientMsg::*;
		for msg in msgs {
			match msg {
				MovePlayerIfSpawned { .. } => (/*already applied locally by control*/),
				AddEffect(effect) => self.handle_add_effect(effect.clone()),
				PlaySound(sound) => self.play_sound(ctx, sound),
				HitPlayer { .. } => (/* handled by server*/),
				ReadyToSpawn => (/*handled by server*/),
				Command(_) => (/*handled by server*/),
			}
		}
	}

	/// Control a player via keyboard/mouse
	fn control_player(&mut self, input_state: &Inputs, dt: f32) {
		let mut clone = self.local_player().clone();
		clone.control(&mut self.pending_diffs, input_state, &self.world, dt);
		*self.local_player_mut() = clone;
	}

	/// Extrapolate other player's positions based on their last know velocity.
	/// This greatly reduces positional stutter in the face of network latency.
	fn extrapolate_other_players(&mut self, dt: f32) {
		for (id, player) in self.world.entities.players.iter_mut() {
			if id != self.local_player_id {
				player.skeleton.position += dt * player.skeleton.velocity;
			}
		}
	}

	/// Animate the players feet if they are moving.
	/// This is done locally by each client (do not send feet position over the network all the time).
	/// Also generate footstep, jump,... sounds locally (do not send these sound effects over the network).
	fn animate_footsteps(&mut self, ctx: &GameCtx, dt: f32) {
		for player_id in self.world.entities.players.copied_ids() {
			let prev = &self.world.entities.players[player_id].local.clone();
			self.world.entities.players[player_id].animate_feet(dt);
			let curr = &self.world.entities.players[player_id].local;
			self.make_footstep_sounds(ctx, player_id, prev, curr);
		}
	}

	fn make_footstep_sounds(&self, ctx: &GameCtx, player_id: ID, prev: &LocalState, curr: &LocalState) {
		let speed = self.world.entities.players[player_id].skeleton.velocity;
		let vspeed = speed.y();
		let walking = { vspeed.abs() < 0.1 && speed != vec3::ZERO };

		if walking {
			if prev.feet_phase.signum() != curr.feet_phase.signum() {
				// make one's own footsteps less loud
				// (quite distracting otherwise)
				let volume = if player_id == self.local_player_id { 0.01 } else { 0.3 };
				self.play_sound_spatial(
					ctx,
					Self::random_footstep_clip(),
					volume,
					&Spatial {
						location: self.world.entities.players[player_id].position(),
					},
				)
			}
		}
	}

	//fn is_on_ground(&self, player_id: ID) -> bool {
	//	let probe = self.world.players[player_id].position() - 0.2 * vec3::EY;
	//	!self.world.map.voxels.at(probe.to_ivec()).is_empty()
	//}

	fn random_footstep_clip() -> &'static str {
		pick_random_clip(&[
			"footstep01", //
			"footstep02",
			"footstep03",
			"footstep04",
			"footstep05",
			"footstep06",
			"footstep07",
			"footstep08",
		])
	}

	//___________________________________________________________________________ effects

	/// Advance visual effects in time.
	/// This is done locally (after creation,
	/// visual effects do not need to synchronize over the network).
	fn tick_effects(&mut self, dt: f32) {
		Self::update_effects_ttl(&mut self.world.entities.effects, dt);
	}

	// decrease effect's TTL by `dt` and remove effects past their TTL.
	fn update_effects_ttl(effects: &mut Vec<Effect>, dt: f32) {
		let mut i = 0;
		while i < effects.len() {
			effects[i].ttl -= dt;
			if effects[i].ttl <= 0.0 {
				effects.swap_remove(i);
			} else {
				i += 1;
			}
		}
	}

	// -------------------------------------------------------------------------------- commands

	pub fn command(&mut self, cmd: &str) -> Result<()> {
		Ok(match &cmd.split_ascii_whitespace().collect::<Vec<_>>()[..] {
			["hello"] => (),
			_ => self.pending_diffs.push(ClientMsg::Command(cmd.into())),
		})
	}

	// __________________________________________________________ accessors

	pub fn world(&self) -> &World {
		&self.world
	}

	/// The player controlled by this client.
	pub fn local_player(&self) -> &Player {
		&self.world.entities.players[self.local_player_id]
	}

	pub fn local_player_mut(&mut self) -> &mut Player {
		&mut self.world.entities.players[self.local_player_id]
	}

	pub fn player_id(&self) -> ID {
		self.local_player_id
	}

	pub fn hud(&self) -> &HUD {
		&self.hud
	}
}

fn azimuth(frame: &Frame, sound_pos: vec3) -> f32 {
	let sound_dir = (sound_pos - frame.position).with(|v| v[Y] = 0.0).normalized();
	let look_dir = frame.orientation.look_dir().with(|v| v[Y] = 0.0).normalized();
	let sin_theta = look_dir.cross(sound_dir).y();
	let cos_theta = look_dir.dot(sound_dir);
	let azimuth = f32::atan2(sin_theta, cos_theta);
	if azimuth.is_nan() {
		0.0
	} else {
		azimuth
	}
}

pub fn pick_random_clip(opts: &[&'static str]) -> &'static str {
	pick_random(opts).unwrap()
}

pub fn pick_random<T>(opts: &[T]) -> Option<&T> {
	match opts.len() {
		0 => None,
		n => Some(&opts[rand::thread_rng().gen_range(0..n)]),
	}
}
