use super::internal::*;

pub struct World {
	// The map is the "static" part of the world -- it never changes.
	// (want a new map, make a new world)
	pub map: Map,

	// The entities are the "dynamic" contents of the world (players, effects, pick-ups).
	// Entities move around, get created and destroyed during gameplay.
	pub entities: Entities,
}

impl World {
	pub fn new(map: Map, entities: Entities) -> Self {
		Self { map, entities }
	}

	/// Intersect a ray (e.g. a line of sight) with the map and players except `player_id`
	/// (to avoid shooting yourself right where the line of fire exits your hitbox).
	/// Returns intersection distance along the ray
	/// and  the ID of the nearest hit player, if any.
	pub fn intersect_except_player(&self, player_id: ID, ray: &Ray64) -> Option<(f64, Option<ID>)> {
		let intersect_map = self.map.intersect_t(ray);
		let mut nearest = intersect_map.map(|t| (t, None));
		for (id, player) in self.entities.players.iter() {
			if let Some(t) = player.intersect(ray) {
				if t < nearest.map(|(t, _)| t).unwrap_or(f64::INFINITY) && id != player_id {
					nearest = Some((t, Some(id)));
				}
			}
		}

		nearest
	}
}
