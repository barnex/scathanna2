use super::internal::*;
use ServerMsg::*;

// World that can only be mutated if diffs are recored.
// (Typestate pattern to avoid accidental mutation without recording diff).
pub struct DiffWorld(World);

impl DiffWorld {
	pub fn new(world: World) -> Self {
		Self(world)
	}

	pub fn apply_to_player_partial<F: Fn(&mut Player)>(&mut self, diffs: &mut Diffs, player_id: ID, f: F) -> Option<()> {
		self.0.entities.players.get_mut(player_id).and_then(|player| {
			Some({
				f(player);
				diffs.push(UpdatePlayerPartial(player.clone()).to_all());
			})
		})
	}

	pub fn force_apply_to_full<F: Fn(&mut Player)>(&mut self, diffs: &mut Diffs, player_id: ID, f: F) -> Option<()> {
		self.0.entities.players.get_mut(player_id).and_then(|player| {
			Some({
				f(player);
				diffs.push(UpdatePlayerFull(player.clone()).to_all());
			})
		})
	}

	pub fn move_player_if_spawned(&mut self, diffs: &mut Diffs, player_id: ID, frame: Frame) {
		if let Some(player) = self.0.entities.players.get_mut(player_id) {
			if player.spawned {
				player.skeleton.set_frame(frame);
				diffs.push(MovePlayer(player_id, player.skeleton.frame()).to_not(player_id));
			}
		}
	}

	pub fn join_new_player(&mut self, diffs: &mut Diffs, spawn_point: &SpawnPoint, join_msg: JoinRequest) -> (ID, MapSwitch) {
		// Join new player cannot be done via apply(msg):
		// we need to add the player before we can get the player ID.

		let player_id = self.0.entities.join_new_player(&spawn_point, join_msg);
		let player = self.entities.players[player_id].clone();
		diffs.push(AddPlayer(player).to_all());

		let map_switch = MapSwitch {
			map_name: self.map.name().into(),
			entities: self.entities.clone(),
		};

		(player_id, map_switch)
	}

	pub fn drop_player(&mut self, diffs: &mut Diffs, player_id: ID) {
		self.0.entities.players.remove(player_id);
		diffs.push(DropPlayer(player_id).to_not(player_id));
	}

	pub fn switch_map(&mut self, diffs: &mut Diffs, new_map: Map) {
		trace!("mapswitch {}", new_map.name());

		let mut tmp = new_map;
		mem::swap(&mut self.0.map, &mut tmp);
		drop(tmp /*now the old map*/);

		let mut players_backup = default();
		mem::swap(&mut self.0.entities.players, &mut players_backup);
		self.0.entities = Entities::default().with(|e| e.players = players_backup);

		diffs.push(
			SwitchMap(MapSwitch {
				map_name: self.0.map.name().into(),
				entities: self.0.entities.clone(),
			})
			.to_all(),
		);
	}
}

impl std::ops::Deref for DiffWorld {
	type Target = World;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
