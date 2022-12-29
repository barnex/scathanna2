use super::internal::*;

pub struct DrawCfg {}

impl Default for DrawCfg {
	fn default() -> Self {
		Self {}
	}
}

impl DrawCfg {
	pub fn draw_gamestate(&self, eng: &GameCtx, zones: &ZoneGraph, viewport_size: uvec2, state: &ClientState) -> SceneGraph {
		let player = state.local_player();
		let world = &state.world;
		let map = world.map.data();
		let hud = &state.hud;

		let mut sg = SceneGraph::new(viewport_size).with(|sg| {
			sg.camera = player.camera();
			sg.bg_color = map.meta.sky_color;
			sg.sun_dir = state.world.map.data().meta.sun_dir;
			sg.sun_color = state.world.map.data().meta.sun_color;
		});

		self.draw_world(eng, &mut sg, zones, world, state.local_player_id);

		hud.draw_on(&mut sg); // TODO: editor does not use resources :(

		sg
	}

	fn draw_world(&self, eng: &GameCtx, sg: &mut SceneGraph, zones: &ZoneGraph, world: &World, local_player_id: ID) {
		zones.draw_on(sg);
		self.draw_players(eng, sg, &world, local_player_id);
		//self.draw_entities(sg, &eng.resources, &world.entities);
		self.draw_effects(sg, &eng.resources, &world.entities.effects);
	}

	fn draw_effects(&self, sg: &mut SceneGraph, rs: &ResourcePack, effects: &Effects) {
		for effect in effects {
			self.draw_effect(sg, rs, effect)
		}
	}

	fn draw_effect(&self, sg: &mut SceneGraph, rs: &ResourcePack, effect: &Effect) {
		match effect.typ {
			EffectType::ParticleExplosion { pos, color } => self.draw_particle_explosion(sg, rs, pos, color, effect.ttl),
			EffectType::ParticleBeam {
				start,
				orientation,
				len,
				color_filter,
			} => self.draw_particle_beam(sg, rs, start, orientation, len, color_filter, effect.ttl),
		};
	}

	fn draw_particle_explosion(&self, sg: &mut SceneGraph, rs: &ResourcePack, pos: vec3, color: vec3, ttl: f32) {
		sg.push(rs.effects.particle_explosion(pos, 1.0 - (ttl / PARTICLE_EXPLOSION_TTL)));
	}

	fn draw_particle_beam(&self, sg: &mut SceneGraph, rs: &ResourcePack, start: vec3, orientation: Orientation, len: f32, color_filter: vec3, ttl: f32) {
		sg.push(rs.effects.particle_beam(start, orientation, len, 1.0 - (ttl / PARTICLE_BEAM_TTL)));
	}

	fn draw_line(&self, sg: &mut SceneGraph, rs: &ResourcePack, start: vec3, end: vec3) {
		let ctx = rs.ctx();
		let buf = MeshBuffer::line(start, end); // TODO: don't upload a new vao each frame
		let vao = Arc::new(ctx.upload_meshbuffer(&buf));
		sg.push(Object::new(&vao, ctx.shader_pack.lines(&ctx.fallback_texture)))
	}

	//fn draw_entities(&self, sg: &mut SceneGraph, rs: &ResourcePack, entities: &Entities) {
	//	for entity in entities.values() {
	//		self.draw_entity(sg, rs, entity)
	//	}
	//}

	//fn draw_entity(&self, sg: &mut SceneGraph, rs: &ResourcePack, entity: &Entity) {
	//	// TODO
	//}

	fn draw_players(&self, eng: &GameCtx, sg: &mut SceneGraph, world: &World, local_player_id: ID) {
		for (_, player) in world.entities.players.iter() {
			if !player.spawned {
				// don't draw players before they spawn
				continue;
			}

			if player.id == local_player_id {
				self.draw_player_1st_person(eng, sg, world, player);
			} else {
				//if camera.can_see(player.position()) {
				self.draw_player_3d_person(eng, sg, world, player);
				//}
			}
		}
	}

	fn draw_player_1st_person(&self, eng: &GameCtx, sg: &mut SceneGraph, _world: &World, player: &Player) {
		//rs.model_pack.get(0 /*TODO*/).draw_1st_person(sg, rs, player);
		//let line_of_fire = player.line_of_fire(world);
		//let shoot_at = player.shoot_at(world);
		//self.draw_line(sg, rs, line_of_fire.start.to_f32(), shoot_at);
	}

	fn draw_player_3d_person(&self, eng: &GameCtx, sg: &mut SceneGraph, world: &World, player: &Player) {
		{
			let matrix = translation_matrix(player.position()) * yaw_matrix(-player.skeleton.frame().orientation.yaw) * scale_matrix(Player::TORSO_HEIGHT);
			let feet_phase = player.local.feet_phase;
			debug_assert!(feet_phase >= -PI && feet_phase <= PI);
			let t = 0.5 * (feet_phase / PI) + 0.5;
			let bob = &eng.resources.animations.bob;
			sg.push(bob.walkingx(&eng.graphics, &eng.resources.animations.heads[player.avatar_id as usize].1, matrix, t));
		}

		{
			let head = eng.resources.animations.heads.get(player.avatar_id as usize).unwrap(/*todo*/);
			let matrix = translation_matrix(player.position() + vec3::EY * Player::TORSO_HEIGHT) //.
				* yaw_matrix(-player.skeleton.frame().orientation.yaw) //.
				* pitch_matrix(0.3 * player.orientation().pitch)
				* scale_matrix(Player::HEAD_HEIGHT);
			sg.push(Object::new(&head.0, eng.graphics.shader_pack.entity(&head.1, matrix)));
		}

		// let line_of_fire = player.line_of_fire(world);
		// let shoot_at = player.shoot_at(world);
		// self.draw_line(sg, rs, line_of_fire.start.to_f32(), shoot_at);
	}
}
