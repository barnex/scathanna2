use super::internal::*;

/// Sampling with anti-aliasing can evaluate points *outside* of a Face.
///
/// This is beneficial for adjacent faces, as it causes their lightmaps to fit seamlessly
/// (sampling only inside a face cases seams like shots/023-weighted-filter.jpg).
///
/// However, sometimes we *want* a seam. E.g. when Faces touch under an angle.
/// otherwise we can get shadow bleeding as shown in shots/023b-shadow-bleeding.jpg.
///
/// To determine if it is fine to sample outside of a Face.
/// we require that the sampling point does not lie *inside* another block,
/// as would happen when two faces make an angle.
/// (This ignores the corner case where a face's edge does not meet up with anything,
/// in which case we get light bleeding but this is almost imperceivable.
/// See shots/024-gauss-filter.jpg)
///
/// TODO: evaluate only once per quadrant around pixel center.
pub fn is_valid_sampling_point(scene: &Scene, pos: vec3) -> bool {
	!scene.block_tree.contains(pos)
}

/// For a face and lightmap image size: pairs of lightmap pixels and corresponding face UV ranges.
/// Corner pixels have UV ranges 1/4 of the normal size, edge pixels 1/2.
///
///   uv
///  (0,0)+---+-------+-------+---+
///       |1/4|  1/2  |       |   |
///       +---+-------+-------+---+
///       |   |  1/1  |       |   |
///       |   |       |       |   |
///       +---+-------+-------+---+
///       |   |       |       |   |
///       |   |       |       |   |
///       +---+-------+-------+---+
///       |   |       |       |   |
///       +---+-------+-------+---+ uv (1,1)
///
pub fn clamped_face_fragments(inner_size: uvec2) -> impl Iterator<Item = (uvec2, Bounds2D<f32>)> {
	cross(0..inner_size.x(), 0..inner_size.y()).map(move |(ix, iy)| {
		let pix = uvec2(ix, iy);
		let (uvmin, uvmax) = clamped_pixel_uv_range(inner_size, pix);
		(pix, Bounds2D::new(uvmin, uvmax))
	})
}

pub fn unclamped_face_fragments(inner_size: uvec2) -> impl Iterator<Item = (uvec2, Bounds2D<f32>)> {
	cross(0..inner_size.x(), 0..inner_size.y()).map(move |(ix, iy)| {
		let pix = uvec2(ix, iy);
		let (uvmin, uvmax) = unclamped_pixel_uv_range(inner_size, pix);
		(pix, Bounds2D::new(uvmin, uvmax))
	})
}

/// Given a pixel index inside a Face's lightmap image (size without margin),
/// return the (u,v) range of points inside that pixel.
///
/// As faces are placed over the lightmap image with a 0.5 pixel offset,
/// corner pixels get only 1/4 of a pixel's surface and edge pixels get 1/2:
///
/// E.g.: here, pixel `a` only gets UV range (0,0) - (0.25,0.25)
/// because 3/4 of it lies outside the face.
/// Pixel `c` gets the full UV range (0.25,0.25) - (0.75,0.75)
/// as it lies fully inside the face.
///     .   .   .   .
///       +-------+
///     . | .   . | .
///       b   c   |
///     . | .   . | .
///       a-------+
///     .   .   .   .
///
/// UVs are in range 0..1.
fn clamped_pixel_uv_range(img_size: uvec2, pix: uvec2) -> (vec2, vec2) {
	let (min, max) = unclamped_pixel_uv_range(img_size, pix);
	// remove the portion of the UV range that lies outside of the face
	let min = min.map(|v| v.clamp(0.0, 1.0));
	let max = max.map(|v| v.clamp(0.0, 1.0));
	(min, max)
}

fn unclamped_pixel_uv_range(img_size: uvec2, pix: uvec2) -> (vec2, vec2) {
	debug_assert!(img_size.iter().all(|v| v > 1));
	debug_assert!(pix.x() < img_size.x() && pix.y() < img_size.y());

	//if !(img_size.iter().all(|v| v > 1)){
	//	return (vec2::ZERO, vec2::ONES)
	//}

	let center_uv = pixel_center_to_uv(img_size, pix.convert());
	let face_size = (img_size - uvec2(1, 1)).to_f32();
	let fragment_size = face_size.map(|v| 1.0 / v);

	let min = center_uv - fragment_size / 2.0;
	let max = center_uv + fragment_size / 2.0;

	(min, max)
}

pub(super) fn pixel_center_to_pos(face: &Face, img_size: uvec2, pix: ivec2) -> vec3 {
	let o = face.origin().to_f32();
	let [a, b] = face.sized_tangents().map(ivec3::to_f32);

	// BUG: should only use for inner pixels
	let (u, v) = pixel_center_to_uv(img_size, pix).into();
	o + (u * a) + (v * b)
}

fn pixel_center_to_uv(img_size: uvec2, pix: ivec2) -> vec2 {
	debug_assert!(img_size.iter().all(|v| v > 1));
	//debug_assert!(pix.x() < img_size.x() && pix.y() < img_size.y());

	let face_size = (img_size - uvec2(1, 1)).to_f32();
	pix.to_f32().div2(face_size)
}
