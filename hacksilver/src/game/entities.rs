use super::internal::*;
use std::sync::atomic::AtomicUsize;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Default)]
pub struct ID(usize);

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

impl std::fmt::Display for ID {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "#{}", self.0)
	}
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Entities {
	// Problematic when we construct new entities for a map switch
	//_next_id: ID,

	// TODO: unify players,entities,effects into Entities
	pub players: Players,
	//pub entities: HashMap<EID, Entity>, // TODO: struct Entities. fn insert(Entity), etc.
	pub effects: Vec<Effect>,
}

impl Entities {
	pub fn join_new_player(&mut self, spawn_point: &SpawnPoint, req: JoinRequest) -> ID {
		let player_id = self.new_id();
		let player = Player::new(player_id, spawn_point.position(), spawn_point.orientation(), req.name, req.avatar_id, req.team);
		self.players.insert(player_id, player);
		player_id
	}

	// A fresh, unique entity number.
	fn new_id(&mut self) -> ID {
		//self._next_id.0 += 1;
		//self._next_id
		ID(NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
	}
}
