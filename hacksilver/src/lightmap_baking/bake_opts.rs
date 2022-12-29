#[derive(Clone, Debug)]
pub struct BakeOpts {
	pub lightmap_resolution: u32,
	pub lightmap_filter_radius: f32,
	pub lightmap_smudge: bool,
	pub lightmap_reflectivity: f32,
	pub lightmap_lamps_samples: u32,
	pub lightmap_sky_samples: u32,
	pub lightmap_indirect_samples: u32,
	//pub lightmap_indirect_depth: u32,
	pub lightmap_bake_normals: bool,
	pub lightmap_sun_only: bool,
	pub lightmap_sky_only: bool,
	pub lightmap_emission_only: bool,
	pub lightmap_scattered_only: bool,
	pub lightmap_ambient_only: bool,
	pub lightmap_nearest: bool,
	pub lightmap_offset: f32,
	pub lightmap_show_validity: bool,
	pub lightmap_outline: bool,
	pub lightmap_blur_sun: u32,
	pub lightmap_blur_all: u32,
	pub lightmap_stitch: bool,
	pub lightmap_error: f32,
}

impl BakeOpts {
	pub const LOW_QUALITY: Self = Self {
		lightmap_resolution: 1,
		lightmap_lamps_samples: 1,
		lightmap_sky_samples: 1,
		lightmap_indirect_samples: 1,
		lightmap_filter_radius: 0.001,
		lightmap_smudge: true,
		lightmap_reflectivity: 0.80,
		//lightmap_indirect_depth: 1,
		lightmap_bake_normals: false,
		//lightmap_visibility_only: false,
		lightmap_sun_only: false,
		lightmap_sky_only: false,
		lightmap_emission_only: false,
		lightmap_scattered_only: false,
		lightmap_ambient_only: false,
		lightmap_nearest: true,
		lightmap_show_validity: false,
		lightmap_offset: 1.0 / 1024.0,
		lightmap_outline: true,
		lightmap_blur_all: 2,
		lightmap_blur_sun: 1,
		lightmap_stitch: true,
		lightmap_error: 0.01,
	};
	pub const MEDIUM_QUALITY: Self = Self {
		lightmap_resolution: 1,
		lightmap_lamps_samples: 10,
		lightmap_sky_samples: 16,
		lightmap_nearest: false,
		lightmap_indirect_samples: 20,
		lightmap_filter_radius: 12.0,
		//lightmap_indirect_depth: 3,
		lightmap_smudge: false,
		lightmap_error: 0.03,
		..Self::LOW_QUALITY
	};
	pub const HIGH_QUALITY: Self = Self {
		lightmap_resolution: 1,
		lightmap_lamps_samples: 64,
		lightmap_sky_samples: 3000,
		lightmap_indirect_samples: 3000,
		lightmap_filter_radius: 12.0,
		lightmap_error: 0.0005,
		//lightmap_indirect_depth: 3,
		lightmap_smudge: false,
		..Self::MEDIUM_QUALITY
	};
}

impl Default for BakeOpts {
	fn default() -> Self {
		Self::LOW_QUALITY.clone()
	}
}
