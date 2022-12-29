use super::internal::*;

/// Vertex struct, copied into WGPU Vertex Buffers.
/// ! `repr(C)` required by WGPU.
/// ! changing field order or adding fields requires `desc()` and shaders to be updated.
///
/// For simplicity, we use one and the same vertex layout for all shaders.
/// Some of the Editor's shaders ignore certain attributes,
/// but gameplay shaders use all of them.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, Default, Serialize, Deserialize)]
pub struct VertexLM {
	pub position: vec3, //
	pub texcoords: vec2,
	pub normal: vec3,
	pub lightcoords: vec2,
	pub tangent_u: vec3,
	pub tangent_v: vec3,
}

impl VertexLM {
	pub const fn new(position: vec3, texcoords: vec2, normal: vec3) -> Self {
		Self {
			position,
			texcoords,
			normal,
			lightcoords: vec2::ZERO,
			tangent_u: vec3::ZERO,
			tangent_v: vec3::ZERO,
		}
	}

	pub const fn new_texcoords(position: vec3, texcoords: vec2) -> Self {
		Self {
			position,
			texcoords,
			normal: vec3::ZERO,
			lightcoords: vec2::ZERO,
			tangent_u: vec3::ZERO,
			tangent_v: vec3::ZERO,
		}
	}

	pub const fn new_normalmap(position: vec3, texcoords: vec2, normal: vec3, tangent1: vec3, tangent2: vec3) -> Self {
		Self {
			position,
			texcoords,
			normal,
			lightcoords: vec2::ZERO,
			tangent_u: tangent1,
			tangent_v: tangent2,
		}
	}

	pub fn transform(&mut self, transf: &mat4) {
		self.position = (transf * self.position.extend(1.0)).xyz();
		self.normal = (transf * self.normal.extend(0.0)).xyz();
	}

	pub fn transformed(&self, transf: &mat4) -> Self {
		self.clone().with(|s| s.transform(transf))
	}

	/// A copy of `self`, with position multiplied by `scale`.
	#[must_use = "Does not modify the original"]
	pub fn map_position<F>(&self, f: F) -> Self
	where
		F: Fn(vec3) -> vec3,
	{
		Self {
			position: f(self.position),
			..self.clone()
		}
	}

	/// Vertex descriptor corresponding to `struct VertexInput` used in *all* our WGSL shaders.
	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
		wgpu::VertexBufferLayout {
			array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &Self::VERTEX_ATTR,
		}
	}

	const VERTEX_ATTR: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
		0 => Float32x3, // position: vec3,
		1 => Float32x2, //texcoords: vec2,
		2 => Float32x3, //normal: vec3,
		3 => Float32x2,  //lightcoords: vec2,
		4 => Float32x3,  //tangent_u: vec3,
		5 => Float32x3,  //tangent_v: vec3,
	];
}
