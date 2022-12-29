use super::internal::*;

// User settings for graphics quality.
// TODO: flags vs. opts (no_msaa vs. msaa)
#[derive(Deserialize, Clone)]
pub struct GraphicsOpts {
	pub width: u32,

	pub height: u32,

	pub fullscreen: bool,

	pub no_msaa: bool,

	pub anisotropy: u8,

	pub texture_resolution: u32,

	pub no_normal_maps: bool,

	pub no_textures: bool,

	pub no_mipmaps: bool,

	pub no_trilinear: bool,

	pub lightmap_nearest: bool,
}

impl GraphicsOpts {
	pub fn msaa_enabled(&self) -> bool {
		!self.no_msaa
	}

	pub fn normal_maps_enabled(&self) -> bool {
		!self.no_normal_maps
	}

	pub fn textures_enabled(&self) -> bool {
		!self.no_textures
	}

	pub fn mipmaps_enabled(&self) -> bool {
		!self.no_mipmaps
	}

	pub fn msaa_sample_count(&self) -> u32 {
		// currently WGPU only supports 1 or 4 samples (https://github.com/gfx-rs/wgpu/issues/1832)
		match self.msaa_enabled() {
			true => 4,
			false => 1,
		}
	}

	pub fn anisotropy_clamp(&self) -> Option<NonZeroU8> {
		match self.anisotropy {
			0 | 1 => None,
			2 | 4 | 8 | 16 => Some(NonZeroU8::new(self.anisotropy).unwrap()),
			_ => None, // invalid. TODO: check on start-up
		}
	}

	pub fn trilinear_enabled(&self) -> bool {
		!self.no_trilinear
	}

	pub fn lightmap_filter(&self) -> &TextureOpts {
		match self.lightmap_nearest {
			true => &NEAREST,
			false => &RGBA_LINEAR,
		}
	}
}

impl Default for GraphicsOpts {
	fn default() -> Self {
		Self {
			width: 1280,
			height: 768,
			fullscreen: false,
			no_msaa: false,
			anisotropy: 16,
			texture_resolution: 512,
			no_normal_maps: false,
			no_textures: false,
			no_mipmaps: false,
			no_trilinear: false,
			lightmap_nearest: false,
		}
	}
}
