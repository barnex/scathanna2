/*
use super::internal::*;

pub struct ModelPack {
	models: [PlayerModel; 1],
}

const HEAD_PITCH_FACTOR: f32 = 0.25;

impl ModelPack {
	// TODO: lazy load?
	pub fn new(ctx: &GraphicsCtx, assets: &AssetsDir) -> Result<Self> {
		Ok(Self {
			models: [
				PlayerModel::frog(ctx, assets)?, //
												 //PlayerModel::panda(engine)?,
												 //PlayerModel::turkey(engine)?,
												 //PlayerModel::pig(engine)?,
												 //PlayerModel::hamster(engine)?,
												 //PlayerModel::chicken(engine)?,
												 //PlayerModel::bunny(engine)?,
			],
		})
	}

	pub fn get(&self, avatar_id: u8) -> &PlayerModel {
		// avatar_id gets checked higher up so should be valid.
		// But just in case, return a default if invalid nevertheless.
		self.models.get(avatar_id as usize).unwrap_or(&self.models[0])
	}
}

/// Models (on the GPU) needed to draw player avatars.
pub struct PlayerModel {
	head: Arc<VAO>,
	foot: Arc<VAO>,
	texture: Arc<Texture>,
	gun: (Arc<VAO>, Arc<Texture>),
	head_height: f32,
	head_scale: f32,
	foot_scale: f32,
	foot_sep: f32,
}

impl PlayerModel {
	pub fn frog(ctx: &GraphicsCtx, assets: &AssetsDir) -> Result<Self> {
		Ok(Self {
			head: wavefront_obj(ctx, assets, "froghead")?,
			foot: wavefront_obj(ctx, assets, "frogfoot")?,
			texture: texture(ctx, assets, "frog")?,
			gun: gun(ctx, assets)?, // TODO: don't duplicate load
			head_height: 4.0,
			head_scale: 8.0,
			foot_scale: 5.0,
			foot_sep: 0.30,
		})
	}

	/// Draw player model as seen by others.
	pub fn draw_3rd_person(&self, sg: &mut SceneGraph, rs: &ResourcePack, player: &Player) {
		self.draw_head(sg, rs, player);
		self.draw_feet(sg, rs, player);
		self.draw_gun(sg, rs, player);

		//if DBG_GEOMETRY {
		//	engine.draw_boundingbox(player.skeleton.bounds());
		//}
	}

	/// Draw player model as seen by self.
	pub fn draw_1st_person(&self, sg: &mut SceneGraph, rs: &ResourcePack, player: &Player) {
		self.draw_feet(sg, rs, player);
		self.draw_gun(sg, rs, player);
	}

	fn draw_gun(&self, sg: &mut SceneGraph, rs: &ResourcePack, player: &Player) {
		let scale_mat = scale_matrix(9.0);
		let Orientation { yaw, pitch } = player.orientation();
		let pitch_mat = pitch_matrix(-pitch);
		let hand_mat = translation_matrix(player.gun_pos_internal());
		let yaw_mat = yaw_matrix(180.0 * DEG - yaw);
		let pos_mat = translation_matrix(player.position());

		let transf = &pos_mat * &yaw_mat * &hand_mat * &pitch_mat * &scale_mat;
		sg.push(Object::new(&self.gun.0, rs.ctx().shader_pack.entity(&self.gun.1, transf)));
	}

	fn draw_head(&self, sg: &mut SceneGraph, rs: &ResourcePack, player: &Player) {
		let Orientation { yaw, pitch } = player.orientation();
		let head_pos = self.head_height * vec3::EY;
		let transf = translation_matrix(player.position() + head_pos) * yaw_matrix(180.0 * DEG - yaw) * pitch_matrix(-pitch * HEAD_PITCH_FACTOR) * scale_matrix(self.head_scale);

		sg.push(Object::new(&self.head, rs.ctx().shader_pack.entity(&self.texture, transf)))
	}

	//pub fn draw_hat(&self, engine: &Engine, player: &Player, hat: &Model) {
	//	let Orientation { yaw, pitch } = player.orientation();
	//	let pitch_mat = pitch_matrix(-pitch * HEAD_PITCH_FACTOR);
	//	let top_mat = translation_matrix((self.head_height + 0.75 * self.head_scale) * vec3::EY);

	//	let yaw_mat = yaw_matrix(-yaw);
	//	let pos_mat = translation_matrix(player.position());
	//	let transf = &pos_mat * &yaw_mat * &pitch_mat * &top_mat;
	//	engine.draw_model_with(hat, &transf);
	//}

	fn draw_feet(&self, sg: &mut SceneGraph, rs: &ResourcePack, player: &Player) {
		let scale_mat = scale_matrix(self.foot_scale);
		let pitch_mat = pitch_matrix(player.local.feet_pitch);
		let [left_mat, right_mat] = self.feet_pos_internal(player).map(translation_matrix);
		let yaw_mat = yaw_matrix(180.0 * DEG - player.orientation().yaw);
		let pos_mat = translation_matrix(player.position());

		let transf_l = &pos_mat * &yaw_mat * &left_mat * &pitch_mat * &scale_mat;
		let transf_r = &pos_mat * &yaw_mat * &right_mat * &pitch_mat * &scale_mat;

		sg.push(Object::new(&self.foot, rs.ctx().shader_pack.entity(&self.texture, transf_l)));
		sg.push(Object::new(&self.foot, rs.ctx().shader_pack.entity(&self.texture, transf_r)));
	}

	fn feet_pos_internal(&self, player: &Player) -> [vec3; 2] {
		let anim_r = 1.0;
		let c = anim_r * player.local.feet_phase.cos();
		let s = anim_r * player.local.feet_phase.sin();
		[
			vec3(-0.35 * player.skeleton.hsize, f32::max(0.0, s), c) - self.foot_sep * vec3::EX,
			vec3(0.35 * player.skeleton.hsize, f32::max(0.0, -s), -c) + self.foot_sep * vec3::EX,
		]
	}
}

fn wavefront_obj(ctx: &GraphicsCtx, assets: &AssetsDir, name: &str) -> Result<Arc<VAO>> {
	Ok(Arc::new(upload_wavefront(ctx, &assets, name)?))
}

fn texture(ctx: &GraphicsCtx, assets: &AssetsDir, name: &str) -> Result<Arc<Texture>> {
	Ok(Arc::new(upload_image(ctx, &assets, name, &ctx.opts.texture_opts())?))
}

fn gun(ctx: &GraphicsCtx, assets: &AssetsDir) -> Result<(Arc<VAO>, Arc<Texture>)> {
	Ok((wavefront_obj(ctx, assets, "bubblegun")?, texture(ctx, assets, "party_hat")?))
}

/*

pub fn parse_avatar_id(s: &str) -> Result<u8> {
	let opts = ["frog", "panda", "turkey", "pig", "hamster", "chicken", "bunny"];
	match s.parse() {
		Ok(v) => Ok(v),
		Err(_) => opts //
			.iter()
			.position(|&v| v == s)
			.map(|v| v as u8)
			.ok_or(anyhow!("avatar options: {}", opts.join(","))),
	}
}




	pub fn panda(engine: &Engine) -> Result<Self> {
		Ok(Self {
			head: engine.wavefront_obj("pandahead")?,
			foot: engine.wavefront_obj("frogfoot")?,
			texture: engine.texture("panda", WHITE),
			gun: gun(engine)?,
			head_height: 1.5,
			head_scale: 4.2,
			foot_scale: 1.6,
			foot_sep: 0.05,
		})
	}

	pub fn pig(engine: &Engine) -> Result<Self> {
		Ok(Self {
			head: engine.wavefront_obj("pighead")?,
			foot: engine.wavefront_obj("simple_foot")?,
			texture: engine.texture("pig", WHITE),
			gun: gun(engine)?,
			head_height: 1.5,
			head_scale: 4.2,
			foot_scale: 1.6,
			foot_sep: 0.05,
		})
	}

	pub fn turkey(engine: &Engine) -> Result<Self> {
		Ok(Self {
			head: engine.wavefront_obj("turkeyhead")?,
			foot: engine.wavefront_obj("chickenleg")?,
			texture: engine.texture("turkey", WHITE),
			gun: gun(engine)?,
			head_height: 1.5,
			head_scale: 4.2,
			foot_scale: 3.0,
			foot_sep: 0.05,
		})
	}

	pub fn hamster(engine: &Engine) -> Result<Self> {
		Ok(Self {
			head: engine.wavefront_obj("hamsterhead")?,
			foot: engine.wavefront_obj("simple_foot")?,
			texture: engine.texture("hamster", WHITE),
			gun: gun(engine)?,
			head_height: 1.5,
			head_scale: 4.2,
			foot_scale: 1.6,
			foot_sep: 0.05,
		})
	}

	pub fn chicken(engine: &Engine) -> Result<Self> {
		Ok(Self {
			head: engine.wavefront_obj("chickenhead")?,
			foot: engine.wavefront_obj("chickenleg")?,
			texture: engine.texture("chicken", WHITE),
			gun: gun(engine)?,
			head_height: 1.5,
			head_scale: 4.2,
			foot_scale: 2.8,
			foot_sep: 0.05,
		})
	}

	pub fn bunny(engine: &Engine) -> Result<Self> {
		Ok(Self {
			head: engine.wavefront_obj("bunnyhead")?,
			foot: engine.wavefront_obj("simple_foot")?,
			texture: engine.texture("bunny", WHITE),
			gun: gun(engine)?,
			head_height: 1.5,
			head_scale: 4.2,
			foot_scale: 1.6,
			foot_sep: 0.05,
		})
	}

}
*/
*/
