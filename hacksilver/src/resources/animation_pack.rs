use super::internal::*;

pub struct AnimationPack {
	pub bob: Animation,
	pub skin: Arc<Texture>,
	pub head: Arc<VAO>,
	pub heads: Vec<(Arc<VAO>, Arc<Texture>)>,
}

impl AnimationPack {
	pub fn new(ctx: &Arc<GraphicsCtx>, assets: &AssetsDir) -> Result<Self> {
		let bob = Animation::load(ctx, assets, "scifi")?;
		let skin = Arc::new(ctx.upload_image_mip(&load_image(&assets, "skin1")?, &default()));
		let head = Arc::new(ctx.upload_meshbuffer(&rescale_head(load_wavefront_merged(&assets, "q_chicken")?)));

		let heads = vec!["bunny", "chicken", "frog", "hamster", "panda", "pig", "turkey"];
		// hack: head modes are 180 degrees rotated :(
		let fix = translation_matrix(vec3(0.0, 0.0, 0.2)) * rotation_matrix(vec3::EY, 180.0 * DEG);
		let heads = heads
			.into_iter()
			.map(|name| -> Result<(Arc<VAO>, Arc<Texture>)> {
				let obj = load_wavefront_merged(assets, name)?;
				let obj = obj.with(|obj| obj.transform(&fix));
				let vao = ctx.upload_meshbuffer(&obj);
				let tex = ctx.upload_image_mip(&load_image(assets, name)?, &TextureOpts::default());
				Ok((Arc::new(vao), Arc::new(tex)))
			})
			.collect::<Result<Vec<_>>>()?;

		Ok(Self { heads, bob, skin, head })
	}
}

pub struct Animation {
	//walk_start: Arc<VAO>,
	walk: Vec<Arc<VAO>>,
	//jump_start: Arc<VAO>,
	//jump: Vec<Arc<VAO>>,
}

impl Animation {
	// Load an animation from `assets/obj/body.*.*.obj`.
	//
	// Walking has 6 keyframes. E.g. for body "torso1":
	// 	`torso1.walk.0.obj`, `torso1.walk.1.obj`,... `torso1.walk.5.obj`
	//
	pub fn load(ctx: &GraphicsCtx, assets: &AssetsDir, body: &str) -> Result<Self> {
		const WALK: &str = "walk";
		const WALK_CYCLE: usize = 6;

		let walk = Self::load_poses(assets, &format!("{body}_{WALK}"), WALK_CYCLE)?;
		let walk = Self::rescale_poses(walk);
		let walk = Self::morph(ctx, &walk)?;

		Ok(Self { walk })
	}

	pub fn walkingx(&self, ctx: &GraphicsCtx, tex: &Arc<Texture>, matrix: mat4, t: f32) -> Object {
		//let t = if t >= 1.0 { 0.0 } else { t };
		assert!(t >= 0.0 && t <= 1.0); // TODO: debug_assert / clamp + warn

		let r = t * (self.walk.len() as f32); // 0.0 .. 6.999..
		let i = r.floor() as usize; // 0 .. 6
		let t = r % 1.0; // 0.0 .. 0.9999...
				 //dbg!(r, i, t);
				 //println!();

		Object {
			vao: self.walk[i].clone(), // TODO: defensive bound check.
			shader: ctx.shader_pack.animation(&tex, matrix, t),
			index_range: None,
		}
	}

	// Load `n` keyframes of an animation cycle.
	// E.g.: cycle_name: `torso1.walk`, n: 6
	fn load_poses(assets: &AssetsDir, cycle_name: &str, n: usize) -> Result<Vec<MeshBuffer>> {
		let poses = (0..n)
			.into_iter()
			.map(|i| format!("{cycle_name}_{i}"))
			.map(|name| load_wavefront_merged(assets, &name))
			.collect::<Result<Vec<_>>>()?;

		//check_indices_per_frame(&poses)?;

		Ok(poses)
	}

	fn rescale_poses(walk: Vec<MeshBuffer>) -> Vec<MeshBuffer> {
		let bounds = BoundingBox::from_points(
			walk.iter() //
				.map(|mesh| mesh.vertices.iter().map(|v| v.position))
				.flatten(),
		)
		.unwrap_or(BoundingBox::new(default(), default()));
		let offset = bounds.min.y() * vec3::EY;
		let scale = 1.0 / bounds.size().y();
		walk.into_iter().map(|mesh| mesh.map_positions(|p| (p - offset) * scale)).collect::<Vec<_>>()
	}

	fn morph(ctx: &GraphicsCtx, poses: &[MeshBuffer]) -> Result<Vec<Arc<VAO>>> {
		let indices = poses[0].indices();

		//let mut morphs = Vec::with_capacity(poses.len());
		//for (i, pose) in poses.iter().enumerate() {
		//	let next = wrap(i + 1, poses.len());
		//	let next = &poses[next];
		//	let host_vertices = Self::morph2(pose, next)?;
		//	let vao = Arc::new(ctx.dev.create_vao(&host_vertices, indices));
		//	morphs.push(vao)
		//}
		//Ok(morphs)

		poses
			.iter()
			.enumerate()
			.map(|(i, pose)| -> Result<Arc<VAO>> {
				let next = &poses[wrap(i + 1, poses.len())];
				let host_vertices = Self::morph2(pose, next)?;
				Ok(Arc::new(ctx.dev.create_vao(&host_vertices, indices)))
			})
			.collect::<Result<Vec<_>>>()
	}

	fn morph2(pose1: &MeshBuffer, pose2: &MeshBuffer) -> Result<Vec<VertexKF>> {
		check_indices_per_frame(&[pose1, pose2])?;

		Ok(pose1
			.vertices()
			.into_iter()
			.zip(pose2.vertices().into_iter())
			.map(|(v1, v2)| VertexKF {
				texcoords: v1.texcoords,
				position1: v1.position,
				position2: v2.position,
				normal1: v1.normal,
				normal2: v2.normal,
			})
			.collect::<Vec<_>>())
	}
}

fn check_indices_per_frame(poses: &[&MeshBuffer]) -> Result<()> {
	for pose in &poses[1..] {
		if pose.vertices.len() != poses[0].vertices.len() {
			return Err(anyhow!("keyframes have different number of vertices"));
		}
		if pose.indices != poses[0].indices {
			return Err(anyhow!("keyframes have different indices"));
		}
	}
	Ok(())
}

fn wrap(i: usize, len: usize) -> usize {
	if i == len {
		0
	} else {
		i
	}
}

fn rescale_head(head: MeshBuffer) -> MeshBuffer {
	let bounds = BoundingBox::from_points(head.vertices.iter().map(|v| v.position)) //
		.unwrap_or(BoundingBox::new(default(), default()));
	let offset = bounds.min.y() * vec3::EY;
	let scale = 1.0 / bounds.size().y();
	head.map_positions(|p| (p - offset) * scale)
}
