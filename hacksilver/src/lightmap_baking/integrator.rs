use super::internal::*;

pub struct Integrator {
	face: Face,
	size: uvec2,
	opts: BakeOpts,
	rng: Xoshiro256PlusPlus,
	img: Img<PixelState>,
	max_samples: u32,
	target_error: f32,
}

struct PixelState {
	halton_seq: HaltonSeq,
	local: Stats,
	area: Stats,
}

impl PixelState {
	pub fn add_sample(&mut self, color: vec3) {
		self.local.add_sample(color)
	}
}

impl Default for PixelState {
	fn default() -> Self {
		let mut rng = rand::thread_rng();
		Self {
			halton_seq: HaltonSeq::new(rng.gen(), rng.gen()),
			local: default(),
			area: default(),
		}
	}
}

impl Integrator {
	pub fn new(face: &Face, opts: &BakeOpts, max_samples: u32, target_error: f32) -> Self {
		let size = lightmap_size_no_margin(opts, &face);
		Self {
			face: face.clone(),
			size,
			opts: opts.clone(),
			rng: Xoshiro256PlusPlus::seed_from_u64(rand::thread_rng().gen()),
			img: Img::new(size),
			max_samples,
			target_error,
		}
	}

	const INITIAL_SAMPLES: usize = 10;

	fn num_samples(state: &PixelState, max_samples: u32, e_want: f32) -> usize {
		let n_have = state.local.n as f32;
		let e_have = bit_error(state);
		let n_want = n_have * e_have / e_want;
		(n_want - n_have).clamp(0.0, max_samples as f32) as usize
	}

	pub fn bake<F>(mut self, scene: &Scene, sample: &F) -> BorderedImg
	where
		F: Fn(&Scene, &HaltonSeq, vec3, Normal) -> Color + Send + Sync,
	{
		self.refine(scene, sample, |_| Self::INITIAL_SAMPLES);

		let max = self.max_samples;
		let err = self.target_error;

		self.update_area_stats();
		self.refine(scene, sample, |state| Self::num_samples(state, max, err) / 4);

		self.update_area_stats();
		self.refine(scene, sample, |state| Self::num_samples(state, max, err));

		self.convert()
	}

	fn refine<F, N>(&mut self, scene: &Scene, sample: &F, num_samples: N)
	where
		F: Fn(&Scene, &HaltonSeq, vec3, Normal) -> Color,
		N: Fn(&PixelState) -> usize,
	{
		let opts = &self.opts;
		let face = &self.face;
		let size = self.size;

		let geom_normal = face.normalized_normal();

		// image snippet (with margin) for this face.

		for (pix, range) in clamped_face_fragments(size) {
			let num_samples = num_samples(&self.img.ref_at(pix));

			for _i in 0..num_samples {
				let uv = (range.min + range.size() * vec2(self.rng.gen(), self.rng.gen())).into();

				// TODO: check if pos inside triangle face
				let pos = face.pos_for_uv(uv);
				let pos = pos + opts.lightmap_offset * geom_normal;

				// TODO: handle case where many samples are invalid
				if is_valid_sampling_point(scene, pos) {
					let shading_normal = scene.shading_normal_for(face, uv);
					let pix = self.img.mut_at(pix);
					let color = sample(scene, &mut pix.halton_seq, pos, shading_normal);
					pix.halton_seq.halton_i += 1;
					pix.add_sample(color);
				}
			}
		}
	}

	fn update_area_stats(&mut self) {
		const SIZE: i32 = 2;
		let size = self.img.size();
		for pix in cross(0..size.x(), 0..size.y()) {
			let pix = uvec2::from(pix);
			let mut area_stats = Stats::default();
			for delta in cross(-SIZE..=SIZE, -SIZE..=SIZE) {
				if let Some(local_stats) = self.img.try_at_i32(pix.to_i32() + ivec2::from(delta)) {
					area_stats.add(&local_stats.local)
				}
			}
			self.img.mut_at(pix).area = area_stats;
		}
	}

	fn convert(self) -> BorderedImg {
		// num samples
		//BorderedImg::from_img_without_margin(self.img.map(|state| {
		//	if state.local.n == Self::INITIAL_SAMPLES {
		//		vec3(0.0, 0.0, 1.0)
		//	} else {
		//		checked_repeat(state.local.n as f32 / 300.0)
		//	}
		//}))

		//BorderedImg::from_img_without_margin(self.img.map(|state| checked_repeat(bit_error(state))))

		//BorderedImg::from_img_without_margin(self.img.map(|state| {
		// checked_repeat(state.stddev())
		//}))

		BorderedImg::from_img_without_margin(self.img.map(|state| state.local.avg()))
	}

	pub fn bake_old<F>(self, scene: &Scene, sample: &F, max_samples: u32) -> BorderedImg
	where
		F: Fn(&Scene, &HaltonSeq, vec3, Normal) -> Color + Send + Sync,
	{
		let opts = &self.opts;
		let face = &self.face;

		let geom_normal = face.normalized_normal();
		// TODO: avoid thread RNG!!
		let mut rng = rand::thread_rng();

		// image snippet (with margin) for this face.
		let mut img = make_image_for_face(&self.opts, face);

		for (pix, range) in clamped_face_fragments(img.inner_size()) {
			let mut rnd = HaltonSeq {
				scramble: vec2(rng.gen(), rng.gen()),
				halton_i: 0,
			};

			let mut acc = Accumulator::new();

			for _i in 0..max_samples {
				let uv = (range.min + range.size() * vec2(rng.gen(), rng.gen())).into();

				// TODO: check if pos inside triangle face
				let pos = face.pos_for_uv(uv);

				let pos = pos + opts.lightmap_offset * geom_normal;

				if is_valid_sampling_point(scene, pos) {
					let shading_normal = scene.shading_normal_for(face, uv);
					let color = sample(scene, &rnd, pos, shading_normal);
					rnd.halton_i += 1;
					acc.add(color);
				}
			}

			img.set_inner_idx(pix, acc.avg().unwrap_or_default());
		}
		img
	}
}

fn bit_error(state: &PixelState) -> f32 {
	let stddev = state.area.stddev() as f32;
	let n = state.local.n as f32;
	if n == 0.0 {
		return 0.0;
	}
	let stderr = stddev / n; // for pseudo-random this would be stddev / sqrt(n), quasi-random is very close to stddev / n;
	let avg = state.area.avg().reduce(f32::add) / 3.0; // TODO: per-color instead of grayscale
	srgb_delta(avg, stderr)
}

fn srgb_delta(center: f32, delta: f32) -> f32 {
	linear_to_srgb_f32(center + delta / 2.0) - linear_to_srgb_f32(center - delta / 2.0)
}

fn checked_repeat(v: f32) -> vec3 {
	if v.is_nan() {
		vec3(1.0, 0.0, 0.0) // RED: DANGER DANGER, NaNs!
	} else {
		vec3::repeat(v as f32) // 255 shades of grey
	}
}
