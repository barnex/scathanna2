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
pub struct VertexKF {
	pub texcoords: vec2,
	pub position1: vec3,
	pub position2: vec3,
	pub normal1: vec3,
	pub normal2: vec3,
}

impl VertexKF {
	/// Vertex descriptor corresponding to `struct VertexInput` used in *all* our WGSL shaders.
	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
		wgpu::VertexBufferLayout {
			array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &Self::VERTEX_ATTR,
		}
	}

	const VERTEX_ATTR: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
		0 => Float32x2,
		1 => Float32x3,
		2 => Float32x3,
		3 => Float32x3,
		4 => Float32x3,
	];
}
