use super::internal::*;

/// Keeps scores and achievements (e.g. double kill).
#[derive(Default)]
pub struct Scores {
	by_player: HashMap<ID, Score>,
	pub by_team: [i32; NUM_TEAMS],
}

#[derive(Default)]
pub struct Score {
	pub total: i32,

	pub frags: u32,
	pub suicides: u32,

	pub multi_kills: u32,
	pub headshots: u32,
	pub deaths: u32,
}

impl Scores {
	pub fn join_new_player(&mut self, id: ID) {
		// make sure player is there with default (zero) score,
		// in case we format scores before the new player scores.
		self.by_player.entry(id).or_default();
	}

	pub fn by_player(&mut self, id: ID) -> &mut Score {
		self.by_player.entry(id).or_default()
	}

	pub fn by_team(&mut self, team: Team) -> &mut i32 {
		&mut self.by_team[team as usize]
	}

	pub fn iter(&self) -> impl Iterator<Item = (ID, &Score)> {
		self.by_player.iter().map(|(&id, score)| (id, score))
	}

	//pub fn top_score(&self) -> i32 {
	//	self.by_player.iter().map(|(_, score)| score.total).max().unwrap_or_default()
	//}

	pub fn reset(&mut self, player_ids: impl Iterator<Item = ID>) {
		*self = default();
		for id in player_ids {
			self.join_new_player(id)
		}
	}

	pub fn max(&self) -> i32 {
		//self.by_player.values().map(|s| s.total).max().unwrap_or_default()
		self.by_team.iter().copied().max().unwrap_or_default()
	}
}
