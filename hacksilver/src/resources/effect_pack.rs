use super::internal::*;

/*

	TODO: unify with Effect

	API:

	EffectPack::draw(Effect).
	  or
	GameCtx::draw(Effect);

	All TTLs etc should be isolated

*/

pub struct EffectPack {
	ctx: Arc<GraphicsCtx>,

	particle_beam: Arc<VAO>,
	particle_explosion: Arc<VAO>,

	pub firefly: Arc<Texture>, // TODO: Arc<texture pack>
}

/// Number of particles per unit of particle beam length.
const PARTICLE_BEAM_DENSITY: u32 = 2;
const PARTICLE_EXPLOSION_N: u32 = 3000;

impl EffectPack {
	pub fn new(ctx: &Arc<GraphicsCtx>, assets: &AssetsDir) -> Result<Self> {
		// TODO: move to texture pack
		let firefly = Arc::new(upload_image(ctx, &assets, "sparkle", &CLAMP_TO_EDGE)?);

		Ok(Self {
			ctx: ctx.clone(),
			particle_beam: Arc::new(ctx.upload_meshbuffer(&Self::particle_beam_vao(ctx))),
			particle_explosion: Arc::new(ctx.upload_meshbuffer(&Self::particle_explosion_vao(ctx, PARTICLE_EXPLOSION_N))),
			firefly,
		})
	}

	pub fn particle_beam(&self, start: vec3, orientation: Orientation, len: f32, phase: f32) -> Object {
		let pitch_mat = pitch_matrix(-90.0 * DEG - orientation.pitch);
		let yaw_mat = yaw_matrix(180.0 * DEG - orientation.yaw);
		let location_mat = translation_matrix(start);
		let transf = location_mat * yaw_mat * pitch_mat;

		// pick the number of triangles to match the desired beam length.
		// number of vertices = 3*number of triangles.
		let vao = &self.particle_beam;
		let n = 3 * (len as u32 + 1) * PARTICLE_BEAM_DENSITY;
		let n = n.clamp(3, vao.num_indices); // TODO!
		let phase = phase.sqrt(); // non-linear progression looks like air drag on the particles
		Object::new(vao, self.ctx.shader_pack.particles(&self.firefly, transf, phase)).with(|o| o.index_range = Some(0..n))
	}

	pub fn particle_explosion(&self, pos: vec3, phase: f32) -> Object {
		let transf = translation_matrix(pos);
		let vao = &self.particle_explosion;
		// decrease number of particles over time
		let phase = 0.8 * phase.sqrt() + 0.2 * phase;
		let n = (((1.0 - phase) * (PARTICLE_EXPLOSION_N as f32)) as u32).clamp(0, PARTICLE_EXPLOSION_N);
		Object::new(vao, self.ctx.shader_pack.particles(&self.firefly, transf, phase)).with(|o| o.index_range = Some(0..n))
	}

	// A VertexArray containing a "particle explosion" consisting of `n` triangles
	// with random orientations, and random velocities pointing away from the origin.
	// To be rendered with `shaders::Particles`.
	fn particle_explosion_vao(ctx: &GraphicsCtx, n: u32) -> MeshBuffer {
		let pos = |_i| vec3(0.0, 0.0, 0.0);
		let vel = |_i| (20.0 + 5.0 * rand::thread_rng().gen::<f32>());
		Self::triangle_particles_vao(ctx, n, pos, vel)
	}

	fn particle_beam_vao(ctx: &GraphicsCtx) -> MeshBuffer {
		let max_dist = 500;
		let n = PARTICLE_BEAM_DENSITY * max_dist;
		let pos = |i| {
			let mut rng = rand::thread_rng();
			let dist = (i as f32) / (PARTICLE_BEAM_DENSITY as f32);
			let rand = vec2(rng.gen(), rng.gen());
			const JITTER: f32 = 0.8;
			let rand = JITTER * uniform_disk(rand);
			vec3(rand.x(), dist, rand.y())
		};
		let vel = |_| 0.7;
		Self::triangle_particles_vao(ctx, n, pos, vel)
	}

	fn triangle_particles_vao(_ctx: &GraphicsCtx, n: u32, pos: impl Fn(usize) -> vec3, vel: impl Fn(usize) -> f32) -> MeshBuffer {
		let mut rng = rand::thread_rng();
		let n = n as usize;

		//let palette = [vec3(1.0, 0.5, 0.5), vec3(1.0, 1.0, 0.5), vec3(0.5, 1.0, 0.5), vec3(0.5, 0.5, 1.0)];

		let mut triangle = MeshBuffer::triangle(&[default(); 3]);
		let mut buf = MeshBuffer::new();

		let tex_coords = [vec2(-0.5, -0.5), vec2(1.5, -0.5), vec2(0.5, 1.5)];

		// TODO: allow y velocity for moving beam
		//const VEL_Y: f32 = 3.0; // hack: extra y-velocity for motion along the beam

		for i in 0..n {
			let norm = sample_isotropic_direction(&mut rng);
			let basis = make_basis(norm);
			let v_dir = sample_isotropic_direction(&mut rng); // + VEL_Y * vec3::EY;
			let vel = vel(i) * v_dir;

			const SCALE: f32 = 3.0;

			for (j, &vert) in TRIANGLE_VERTICES.iter().enumerate() {
				triangle.vertices[j].texcoords = tex_coords[j];
				triangle.vertices[j].position = basis * vert * SCALE + pos(i);
				triangle.vertices[j].normal = vel; // !! hack: reusing normal as velocity :(
			}

			buf.append(&triangle)
		}

		buf
	}
}

// Vertices of an equilateral triangle centered at (0,0,0).
// "Prototype" for all particle triangles.
const TRIANGLE_VERTICES: [vec3; 3] = [
	vec3(-0.5, -SIN_60 / 2.0, 0.0), //.
	vec3(0.5, -SIN_60 / 2.0, 0.0),
	vec3(0.0, SIN_60 / 2.0, 0.0),
];

const SIN_60: f32 = 0.86602540378;

/// Sample a random unit vector with isotropically distributed direction.
fn sample_isotropic_direction(rng: &mut impl rand::Rng) -> vec3 {
	let norm = rand_distr::StandardNormal;
	vec3(rng.sample(norm), rng.sample(norm), rng.sample(norm)).normalized()
}
