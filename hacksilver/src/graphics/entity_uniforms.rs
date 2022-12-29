/*
use super::internal::*;
pub(super) struct InstanceBuffer {
	pub buffer: wgpu::Buffer,
	//pub bind_group: wgpu::BindGroup,
}

// https://sotrh.github.io/learn-wgpu/beginner/tutorial7-instancing/#the-instance-buffer
impl InstanceBuffer {
	pub fn new(device: &wgpu::Device) -> Self {
		let hostdata = [mat4::ZERO; MAX_ENTITIES]; // Vec should work too. (?).
		let buffer = device.create_buffer_init(
			//
			&wgpu::util::BufferInitDescriptor {
				label: Some(file!()),
				contents: bytemuck::cast_slice(&[hostdata]),
				usage: wgpu::BufferUsages::VERTEX, // ! different from Camera
			},
		);

		// let bind_group_layout = Self::bind_group_layout(device);
		// let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
		// 	layout: &bind_group_layout,
		// 	entries: &[wgpu::BindGroupEntry {
		// 		binding: 0,
		// 		resource: buffer.as_entire_binding(),
		// 	}],
		// 	label: Some(file!()),
		// });

		Self { buffer }
	}

	// pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
	// 	device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
	// 		entries: &[wgpu::BindGroupLayoutEntry {
	// 			binding: 0,
	// 			visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
	// 			ty: wgpu::BindingType::Buffer {
	// 				ty: wgpu::BufferBindingType::Uniform,
	// 				has_dynamic_offset: false,
	// 				min_binding_size: None,
	// 			},
	// 			count: None,
	// 		}],
	// 		label: Some(file!()),
	// 	})
	// }
}
*/
