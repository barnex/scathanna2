use super::internal::*;

/// Data buffer to be uploaded as global uniform data (shaders: `struct Globals`).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct GlobalsHostData {
	// global camera
	view_proj: [[f32; 4]; 4],
	cam_position: vec3,

	// global sun_direction
	sun_dir: vec3,
	sun_color: vec3,

	_padding: [f32; 3], // be still, my wgpu.
}

impl GlobalsHostData {
	pub fn from(camera: &Camera, viewport_size: uvec2, sun_dir: vec3, sun_color: vec3) -> Self {
		Self {
			view_proj: camera.matrix(viewport_size),
			cam_position: camera.position,
			sun_dir,
			sun_color,
			_padding: default(),
		}
	}
}

pub(super) struct GlobalUniforms {
	pub buffer: wgpu::Buffer,
	pub bind_group: wgpu::BindGroup,
}

impl GlobalUniforms {
	pub fn new(device: &wgpu::Device) -> Self {
		let hostdata = GlobalsHostData::default();
		let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some(file!()),
			contents: bytemuck::cast_slice(&[hostdata]),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		});

		let bind_group_layout = Self::bind_group_layout(device);

		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &bind_group_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: buffer.as_entire_binding(),
			}],
			label: Some(file!()),
		});
		Self { buffer, bind_group }
	}

	pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries: &[wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			}],
			label: Some(file!()),
		})
	}
}
