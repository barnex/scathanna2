use super::internal::*;
use Addressee::*;
use ServerMsg::*;

pub struct GameLogic {
	assets: AssetsDir,
	maplist: Vec<String>,
	curr_map: usize,
	scores: Scores,
	sprees: HashMap<ID, (f32, u32)>,
	frag_limit: i32,
	time_playing: f32,
	time_limit: f32,
}

const SPREE_TIME: f32 = 3.0;

impl GameLogic {
	pub fn new(assets: AssetsDir, settings: ServerOpts) -> Result<Self> {
		println!("server: maplist: {}", settings.maplist.join(", "));
		let maplist = match settings.maplist.len() {
			0 => assets.find_all_maps()?,
			_ => settings.maplist.clone(),
		};
		verify_maps(&assets, &maplist)?;

		Ok(Self {
			assets,
			maplist,
			curr_map: 0,
			scores: default(),
			frag_limit: settings.frag_limit as i32,
			time_limit: settings.time_limit as f32,
			time_playing: 0.0,
			sprees: default(),
		})
	}

	//-------------------------------------------------------------------------------- tick

	pub fn tick(&mut self, data: &mut ServerData, dt: f32) {
		self.tick_time_remaining(data, dt);
		self.tick_next_game(data);

		self.tick_lava(data);
		self.tick_killplane(data);
	}

	fn tick_time_remaining(&mut self, data: &mut ServerData, dt: f32) {
		self.time_playing += dt;

		if (self.time_playing + dt) as i32 != self.time_playing as i32 {
			// broadcast every second so the timer ticks down
			self.broadcast_scores(data);
		}
		// 1 minute warning
		if (self.time_remaining() + dt) > 60.00 && self.time_remaining() <= 60.0 {
			trace!("1 minute warning");
			data.sound_announce(All, "ann_1_minute_warning");
		}
	}

	fn tick_next_game(&mut self, data: &mut ServerData) {
		if self.time_playing > self.time_limit {
			self.switch_next_map(data);
		}

		if self.scores.max() >= self.frag_limit as i32 {
			self.switch_next_map(data);
		}
	}

	fn announce_winner(&mut self, data: &mut ServerData) {
		use Team::*;
		let top_score = self.scores.max();
		let winning_team = [Red, Green, Blue].into_iter().find(|&t| *self.scores.by_team(t) == top_score);
		if let Some(winning_team) = winning_team {
			data.hud_announce(All, format!("Team {winning_team} wins!"));
			data.sound_announce(
				All,
				match winning_team {
					Red => "ann_red_wins",
					Green => "ann_green_wins",
					Blue => "ann_blue_wins",
				},
			);
		}

		let sorted_teams = vec![Red, Green, Blue].with(|v| v.sort_by_key(|&team| self.scores.by_team[team as usize])).with(|v| v.reverse());
		//let sorted_players = data.players().collect::<Vec<_>>().with(|v|v.sort_by_key(|id|data.player(id).map()))
		use std::fmt::Write;
		let mut scores = String::new();
		for team in sorted_teams {
			let _ = writeln!(&mut scores, "\n\nTeam {team}");
			let _ = writeln!(&mut scores, "___________________________________________\n");
			for id in data.players() {
				if data.player(id).map(|p| p.team) == Some(team) {
					let score = self.scores.by_player(id);
					let _ = writeln!(&mut scores, "{:+20}: {:2} frags | {:2} deaths", data.must_name(id), score.frags, score.deaths);
				}
			}
		}
		println!("{}", &scores);
		data.hud_announce2(All, scores);
	}

	pub fn switch_next_map(&mut self, data: &mut ServerData) {
		info!("switching to next map");
		self.curr_map += 1;
		if self.curr_map >= self.maplist.len() {
			self.curr_map = 0;
		}
		self.switch_map(data, self.curr_map).expect("previously validated map failed to load")
	}

	// lava system kills players who are on lava
	fn tick_lava(&mut self, data: &mut ServerData) {
		for id in data.spawned_player_ids() {
			(|| {
				Some({
					// Super hack to determine if we are on lava.
					// TODO: reserved material IDs, or mark properties (lava, water, translucent,...) in palette.
					// TODO: why does the player hover 1 unit above the ground (round to int physics??).
					let start = data.player(id)?.position();
					let dir = -vec3::EY;
					let ray = Ray::new(start, dir);
					let hit = data.world.map.intersect(&ray);
					if hit.t < 2.0 {
						if let Some((_, _, mat_id)) = hit.attrib {
							if data.world.map.data().palette.material_name_for(mat_id).unwrap_or_default().starts_with("Lava") {
								self.suicide(data, id, "fell in lava");
							}
						}
					}
				})
			})();
		}
	}

	// kill players who fell off the world
	fn tick_killplane(&mut self, data: &mut ServerData) {
		const KILL_PLANE: f32 = -512.0;
		for id in data.spawned_player_ids() {
			(|| {
				Some({
					if data.player(id)?.position().y() < KILL_PLANE {
						self.suicide(data, id, "fell off the world")?;
					}
				})
			})();
		}
	}

	//-------------------------------------------------------------------------------- scoring

	// Handle a client saying they just shot a player.
	// We trust clients not to lie about this.
	//
	// Hitting players is computed client-side for latency reasons:
	// a client always sees other players at a location that lags slightly behind.
	// If a client hits a player where they see them on their screen, then it should
	// count as a hit regardless of latency.
	// Otherwise players with more than about 30ms latency would be at a noticeable disadvantage.
	pub fn handle_hit_player(&mut self, data: &mut ServerData, actor: ID, victim: ID) -> Option<()> {
		trace!("{actor} hit {victim}");

		self.active_kill(data, actor, victim)?;

		Some(())
	}

	pub fn suicide(&mut self, data: &mut ServerData, victim: ID, msg: &str) -> Option<()> {
		if data.player(victim)?.spawned {
			trace!("{victim} suicide");
			self.scores.by_player(victim).total -= 1;
			self.scores.by_player(victim).suicides += 1;
			self.passive_kill(data, victim);
			data.log(format!("{} {}", data.must_name(victim), msg));
			data.hud_announce(Just(victim), format!("You {}", msg));
			data.sound_announce(Just(victim), "ann_be_careful");
		}

		Some(())
	}

	/// Someone killed someone else
	fn active_kill(&mut self, data: &mut ServerData, actor: ID, victim: ID) -> Option<()> {
		let actor_team = data.player(actor)?.team;
		let vicitm_team = data.player(victim)?.team;

		if actor_team == vicitm_team {
			trace!("friendly fire {actor} -> {victim}");
			return None;
		}

		trace!("{actor} killed {victim}");

		//  "N frags remain gets announced when the leader makes progress"
		let remaining1 = self.scores.max() - self.frag_limit;

		*self.scores.by_team(actor_team) += 1;
		self.scores.by_player(actor).frags += 1;
		self.record_spree(data, actor);

		let remaining2 = self.scores.max() - self.frag_limit;
		if remaining1 != remaining2 {
			self.announce_remaining_frags(data)
		}

		data.log(format!("{} fragged {}", data.must_name(actor), data.must_name(victim)));
		data.hud_announce(Just(actor), format!("You fragged {}", data.must_name(victim)));
		data.hud_announce(Just(victim), format!("You got fragged by {}", data.must_name(actor)));

		self.passive_kill(data, victim);

		Some(())
	}

	fn record_spree(&mut self, data: &mut ServerData, player: ID) {
		dbg!(&self.sprees);

		if !self.sprees.contains_key(&player) {
			self.sprees.insert(player, (self.time_playing, 1));
			return;
		}

		if let Some(&entry) = self.sprees.get(&player) {
			if (self.time_playing - entry.0) / (entry.1 as f32) < SPREE_TIME {
				let entry = entry.with(|e| e.1 += 1);
				let n = entry.1;
				self.sprees.insert(player, entry);
				data.hud_announce2(
					Just(player),
					match n {
						0 | 1 => "".into(),
						2 => "Double frag!".into(),
						3 => "Multi frag!".into(),
						4 => "Incredible frag!".into(),
						5 => "Unstoppable!!".into(),
						n => format!("{n} frags in a row!!!"),
					},
				);
				match n {
					0 | 1 => (), // unreachable!(),
					2 => data.sound_announce(Just(player), "ann_double_frag"),
					3 => data.sound_announce(Just(player), "ann_multi_frag"),
					4 => data.sound_announce(Just(player), "ann_incredible"),
					_ => data.sound_announce(Just(player), "ann_unstoppable"),
				}
			} else {
				self.sprees.remove(&player);
			}
		}
	}

	fn announce_remaining_frags(&mut self, data: &mut ServerData) {
		let top_score = self.scores.max();
		if top_score == (self.frag_limit as i32) - 1 {}

		let remaining = self.frag_limit as i32 - top_score;
		info!("{remaining} frag(s) remaining");
		match remaining {
			1 => data.sound_announce(All, "ann_1_frag_remains"),
			2 => data.sound_announce(All, "ann_2_frags_remain"),
			3 => data.sound_announce(All, "ann_3_frags_remain"),
			_ => (),
		}
	}

	fn passive_kill(&mut self, data: &mut ServerData, victim: ID) -> Option<()> {
		data.despawn(victim)?;
		data.add_effect(Effect::particle_explosion(data.player(victim)?.center(), WHITE));
		self.broadcast_scores(data);
		self.sprees.remove(&victim);
		Some(())
	}

	fn broadcast_scores(&mut self, data: &mut ServerData) {
		// Score delta:
		// 	`+N` against the second one if you're leading,
		//  `-N` against the leader if you're behind.
		let sorted = sorted(self.scores.by_team.to_vec()).with(|v| v.reverse());
		let top_score = sorted.get(0).copied().unwrap_or_default();
		let scnd_score = sorted.get(1).copied().unwrap_or_default();
		let delta = |score| if score == top_score { score - scnd_score } else { score - top_score };
		let max = self.frag_limit;

		let sec_remaining = f32::max(0.0, self.time_remaining()) as u32;
		let min = sec_remaining / 60;
		let sec = sec_remaining % 60;

		for (id, _score) in self.scores.iter() {
			let team = match data.player(id) {
				None => continue,
				Some(player) => player.team,
			};
			let score = self.scores.by_team[team as usize];
			let delta = delta(score);
			let text = format!("time: {min}:{sec:02}\n{team}: {score} / {max} ({delta:+})");

			data.push_no_apply(
				UpdateHUD(HUDUpdate {
					pos: HUDPos::TopLeft,
					text,
					ttl_sec: INF,
				})
				.to_just(id),
			);
		}
	}

	fn time_remaining(&self) -> f32 {
		self.time_limit - self.time_playing
	}

	//-------------------------------------------------------------------------------- respawn

	pub fn handle_ready_to_respawn(&mut self, data: &mut ServerData, player_id: ID) {
		trace!("ready_to_respawn: {player_id}");
		let spawn_point = self.pick_spawn_point(&data.world);
		data.apply_to_player_full(player_id, |p| {
			if !p.spawned {
				trace!("respawn {player_id}");
				// client could request spawn multiple times in a network race.
				p.spawned = true;
				p.skeleton.position = spawn_point.position();
				p.skeleton.orientation = spawn_point.orientation();
				p.invulnerability_ttl = self.invul_ttl(); // spawn kill protection
			}
		});
	}

	fn pick_spawn_point(&self, world: &World) -> SpawnPoint {
		pick_random(&world.map.data().meta.spawn_points).cloned().unwrap_or_default()
	}

	/// Invulnerable seconds after spawn.
	fn invul_ttl(&self) -> Option<f32> {
		Some(1.5)
	}

	//-------------------------------------------------------------------------------- join/drop/switch players

	pub fn join_new_player(&mut self, data: &mut ServerData, join_msg: JoinRequest) -> (ID, MapSwitch) {
		let spawn_point = self.pick_spawn_point(&data.world);
		let (player_id, map_switch) = data.join_new_player(&spawn_point, join_msg);
		self.scores.join_new_player(player_id);
		self.broadcast_scores(data);
		(player_id, map_switch)
	}

	pub fn drop_player(&mut self, _player_id: ID) {}

	fn switch_map(&mut self, data: &mut ServerData, map_idx: usize) -> Result<()> {
		for id in data.players() {
			data.despawn(id);
		}

		self.announce_winner(data);

		let map_name = &self.maplist.get(map_idx).ok_or(bug())?;
		let new_map = Map::load(&self.assets, map_name)?;

		data.switch_map(new_map);
		self.curr_map = map_idx;

		self.time_playing = 0.0;
		self.scores.reset(data.players());
		self.broadcast_scores(data);

		data.hud_announce(All, self.curr_map_name().to_owned());

		Ok(())
	}

	// name of the currently active map
	pub fn curr_map_name(&self) -> &str {
		&self.maplist.get(self.curr_map).map(String::as_str).unwrap_or("???")
	}

	//-------------------------------------------------------------------------------- commands

	/// Respond to text command "switch my_map".
	pub fn switch_map_cmd(&mut self, data: &mut ServerData, map_name: &str) -> Result<()> {
		let map_idx = self
			.maplist
			.iter()
			.position(|x| x == map_name)
			.ok_or_else(|| anyhow!("no such map: `{}`, options: {:?}", map_name, &self.maplist))?;

		self.switch_map(data, map_idx)
	}
}

fn verify_maps(assets: &AssetsDir, maplist: &[String]) -> Result<()> {
	if maplist.len() == 0 {
		return Err(anyhow!("server: maplist: need at least one map"));
	}
	for map_name in maplist {
		if let Err(e) = Map::load(assets, map_name) {
			return Err(anyhow!("map {} failed verification: {}", map_name, e));
		}
	}
	Ok(())
}

fn bug() -> Error {
	anyhow!("BUG")
}

fn todo() -> Error {
	anyhow!("TODO")
}

const WHITE: vec3 = vec3(1.0, 1.0, 1.0);

//fn by_chance(probabilty: f32) -> bool {
//	rand::thread_rng().gen::<f32>() < probabilty
//}
