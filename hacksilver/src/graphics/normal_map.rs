use super::internal::*;

// Convert a normalmap image in linear space (not SRGB)
// to normalized vectors in tangent space.
// See https://en.wikipedia.org/wiki/Normal_mapping#Calculation
pub fn decode_normal_map(normal_map: &RgbImage) -> Img<vec3> {
	let mut img = Img::new(normal_map.dimensions().into());

	for (x, y, &Rgb([r, g, b])) in normal_map.enumerate_pixels() {
		let rgb = Vector3::new(r, g, b).map(|v| (v as f32) / 255.0); // map to 0..1
		let normal = 2.0 * (rgb - vec3(0.5, 0.5, 0.5)); // map to -1..1
		let normal = normal.normalized(); // mitigate small truncation errors
		img.set((x, y), normal);
	}

	img
}

// Inverse of `decode_normal_map`:
// encode normal vectors in linear RGB (not SRGB).
pub fn encode_normal_map(normal_map: &Img<vec3>) -> RgbImage {
	let (w, h) = normal_map.size().into();
	RgbImage::from_fn(w, h, |x, y| {
		let normal = normal_map.at((x, y));
		let rgb = 0.5 * (normal + vec3(1.0, 1.0, 1.0)); // map to 0..1
		let rgb_linear_u8 = rgb.map(|v| (v * 255.0).clamp(0.0, 255.0) as u8);
		Rgb(rgb_linear_u8.into())
	})
}
