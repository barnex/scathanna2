use super::internal::*;

// UV-mapping for plane defined by normal.
pub fn uv_basis(face: &Face) -> mat3 {
	// obscure fiddling to make textures reasonably fit.
	const FIDDLE: f32 = 0.4;
	let normal = face.normalized_normal().map(|v| if v.abs() < FIDDLE { 0.0 } else { v }).normalized();

	if normal.y() == 0.0 {
		// Vertical-ish walls
		let up = -vec3::EY;
		let horiz = normal.cross(up).normalized();
		let normal = horiz.cross(up).normalized();
		let up = normal.cross(horiz).normalized();
		Matrix3([horiz, up, normal])
	} else {
		// Horizontal-ish floors and ceilings
		let vertical = normal;
		let tangent = vec3::EX;
		let bitangent = tangent.cross(vertical).normalized();
		let tangent = vertical.cross(bitangent).normalized();
		let normal = tangent.cross(bitangent).normalized();
		Matrix3([tangent, bitangent, normal])
	}
}

pub fn uv_project(basis: &Matrix3<f32>, pos: vec3) -> vec2 {
	//let proj = basis * pos;
	let u = basis[0].dot(pos);
	let v = basis[1].dot(pos);
	let uv = vec2(u, v);
	let pitch = 1.0 / 64.0; // TODO
	let uv = uv * pitch;
	let uv = uv.map(|v| v % 256.0); // modulo so that uv coordinates don't grow too big.
	uv
}
