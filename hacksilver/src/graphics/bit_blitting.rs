use super::internal::*;

/// A rectangular mesh for blitting (pixel-perfect copying) from a texture to screen space (WGPU/Vulcan coordinates).
///
/// tex_pixels         screen_pixels
/// +--------+      +------------------+
/// |tex +-+ |      | scrn +-+         |
/// |pos |\| |      |  pos |\|         |
/// |    +-+ |      |      +-+         |
/// +--------+      |                  |
///                 +------------------+
pub fn blit(tex_pixels: uvec2, tex_pos: uvec2, sprite_pixels: uvec2, scrn_pixels: uvec2, scrn_pos: uvec2) -> MeshBuffer {
	let sprite_pixels = sprite_pixels.to_f32();
	let tex_pixels = tex_pixels.to_f32();

	// UV position in the source texture (0..1)
	let uv0 = tex_pos.to_f32() / tex_pixels;
	let uv_size = sprite_pixels / tex_pixels;
	let uv1 = uv0 + uv_size;

	let scrn_pixels = scrn_pixels.to_f32();
	let scrn_pos = scrn_pos.to_f32();

	let dst_x0 = linterp(0.0, -1.0, scrn_pixels.x(), 1.0, scrn_pos.x());
	let dst_y0 = linterp(0.0, 1.0, scrn_pixels.y(), -1.0, scrn_pos.y());
	let dst_x1 = linterp(0.0, -1.0, scrn_pixels.x(), 1.0, scrn_pos.x() + sprite_pixels.x());
	let dst_y1 = linterp(0.0, 1.0, scrn_pixels.y(), -1.0, scrn_pos.y() + sprite_pixels.y());

	let z = 0.0;
	let vertices = [
		VertexLM::new_texcoords(vec3(dst_x0, dst_y0, z), vec2(uv0.x(), uv0.y())),
		VertexLM::new_texcoords(vec3(dst_x1, dst_y0, z), vec2(uv1.x(), uv0.y())),
		VertexLM::new_texcoords(vec3(dst_x1, dst_y1, z), vec2(uv1.x(), uv1.y())),
		VertexLM::new_texcoords(vec3(dst_x0, dst_y1, z), vec2(uv0.x(), uv1.y())),
	];

	MeshBuffer::rect(&vertices)
}
