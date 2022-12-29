use super::internal::*;
use mem::size_of;

/// Instancing matrix struct, copied into WGPU Instance Buffers.
/// ! `repr(C)` required by WGPU.
/// ! changing field order or adding fields requires `desc()` and shaders to be updated.
///
/// See https://sotrh.github.io/learn-wgpu/beginner/tutorial7-instancing/#the-instance-buffer.
#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Pod, Zeroable)]
pub struct InstanceRaw {
	pub model_matrix: [[f32; 4]; 4],
	pub extra: [f32; 2],
}

impl InstanceRaw {
	// Create a buffer of `InstanceRaw`s on the device.
	pub fn new_buffer(device: &wgpu::Device, data: &[InstanceRaw]) -> wgpu::Buffer {
		device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some(file!()),
			contents: bytemuck::cast_slice(data),
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
		})
	}

	/// Instance descriptor corresponding to `struct InstanceInput` used in `entity.wgsl`.
	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
		const VEC4: usize = size_of::<vec4>();
		use wgpu::BufferAddress;
		use wgpu::VertexAttribute;
		use wgpu::VertexFormat::*;

		wgpu::VertexBufferLayout {
			array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
			// Our shaders will only change to use the next
			// instance when the shader starts processing a new instance
			step_mode: wgpu::VertexStepMode::Instance, // !
			attributes: &[
				VertexAttribute {
					offset: (0 * VEC4) as BufferAddress,
					shader_location: 6, // entity.wgsl: model_matrix_0
					format: Float32x4,
				},
				VertexAttribute {
					offset: (1 * VEC4) as BufferAddress,
					shader_location: 7, // entity.wgsl: model_matrix_1
					format: Float32x4,
				},
				VertexAttribute {
					offset: (2 * VEC4) as BufferAddress,
					shader_location: 8, // entity.wgsl: model_matrix_2
					format: Float32x4,
				},
				VertexAttribute {
					offset: (3 * VEC4) as BufferAddress,
					shader_location: 9, // entity.wgsl: model_matrix_3
					format: Float32x4,
				},
				VertexAttribute {
					offset: (4 * VEC4) as BufferAddress,
					shader_location: 10, // entity.wgsl: extra
					format: Float32x2,
				},
			],
		}
	}
}
