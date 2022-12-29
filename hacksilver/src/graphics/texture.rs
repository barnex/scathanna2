use super::internal::*;

/// High-level wrapper around a WGPU Texture, View and Sampler
pub struct Texture {
	pub texture: wgpu::Texture,
	pub view: wgpu::TextureView,
	pub sampler: wgpu::Sampler,
}

/// A 1x1 texture filled with a uniform color.
pub fn uniform_texture(ctx: &GraphicsCtx, color: vec4) -> Texture {
	let (r, g, b) = color.xyz().map(linear_to_srgb).into();
	let a = (color.w().clamp(0.0, 1.0) * 255.0) as u8;
	ctx.upload_rgba(&[r, g, b, a], uvec2(1, 1), &NEAREST)
}
