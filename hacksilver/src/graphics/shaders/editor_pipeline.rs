use super::internal::*;

pub struct EditorPipeline {
	pub pipeline: wgpu::RenderPipeline,
	texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl EditorPipeline {
	pub fn new(opts: &GraphicsOpts, device: &wgpu::Device, surface_format: wgpu::TextureFormat, camera_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
		let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some(file!()),
			source: wgpu::ShaderSource::Wgsl(include_str!("editor.wgsl").into()),
		});

		let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries: &[
				// Binding 0: texture data
				wgpu::BindGroupLayoutEntry {
					binding: 0, // Fragment shader: t_diffuse;
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						view_dimension: wgpu::TextureViewDimension::D2,
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
					},
					count: None,
				},
				// Binding 1: texture sampler
				wgpu::BindGroupLayoutEntry {
					binding: 1, // Fragment shader: s_diffuse;
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
			],
			label: Some(file!()),
		});

		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some(file!()),
			layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some(file!()),
				bind_group_layouts: &[&texture_bind_group_layout, camera_bind_group_layout],
				push_constant_ranges: &[],
			})),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[VertexLM::desc()],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState {
					format: surface_format,
					blend: Some(wgpu::BlendState {
						color: wgpu::BlendComponent::OVER,
						alpha: wgpu::BlendComponent::OVER,
					}),
					write_mask: wgpu::ColorWrites::ALL,
				})],
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Cw,
				cull_mode: Some(wgpu::Face::Front),
				polygon_mode: wgpu::PolygonMode::Fill,
				unclipped_depth: false,
				conservative: false,
			},
			depth_stencil: Some(wgpu::DepthStencilState {
				format: Canvas::DEPTH_FORMAT,
				depth_write_enabled: true,
				depth_compare: wgpu::CompareFunction::Less,
				stencil: wgpu::StencilState::default(),
				bias: wgpu::DepthBiasState::default(),
			}),
			multisample: wgpu::MultisampleState {
				count: opts.msaa_sample_count(),
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			multiview: None,
		});

		Self { pipeline, texture_bind_group_layout }
	}

	fn texture_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
		&self.texture_bind_group_layout
	}

	pub fn texture_bind_group(&self, dev: &DeviceCtx, texture: &Texture) -> wgpu::BindGroup {
		dev.device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: self.texture_bind_group_layout(),
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0, //
					resource: wgpu::BindingResource::TextureView(&texture.view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&texture.sampler),
				},
			],
			label: Some(file!()),
		})
	}
}
