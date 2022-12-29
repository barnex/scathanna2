use super::internal::*;

/// A shader
pub struct NormalmapPipeline {
	pub pipeline: wgpu::RenderPipeline,
	texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl NormalmapPipeline {
	pub fn new(opts: &GraphicsOpts, device: &wgpu::Device, surface_format: wgpu::TextureFormat, camera_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
		let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some(file!()),
			source: wgpu::ShaderSource::Wgsl(include_str!("normalmap.wgsl").into()),
		});

		let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries: &[
				// diffuse texture
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
				wgpu::BindGroupLayoutEntry {
					binding: 1, // Fragment shader: s_diffuse;
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
				// lightmap texture
				wgpu::BindGroupLayoutEntry {
					binding: 2, // Fragment shader: t_lightmap
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						view_dimension: wgpu::TextureViewDimension::D2,
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 3, // Fragment shader: s_lightmap
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
				// normal texture
				wgpu::BindGroupLayoutEntry {
					binding: 4, // Fragment shader: t_normalmap
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						view_dimension: wgpu::TextureViewDimension::D2,
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 5, // Fragment shader: s_lightmap
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
				// direct light visibility
				wgpu::BindGroupLayoutEntry {
					binding: 6, // Fragment shader: t_direct
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						view_dimension: wgpu::TextureViewDimension::D2,
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 7, // Fragment shader: s_direct
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
						color: wgpu::BlendComponent::REPLACE,
						alpha: wgpu::BlendComponent::REPLACE,
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

	pub fn texture_bind_group(&self, dev: &DeviceCtx, texture: &Texture, lightmap: &Texture, normalmap: &Texture, direct: &Texture) -> wgpu::BindGroup {
		use wgpu::BindGroupEntry;
		use wgpu::BindingResource;

		dev.device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: self.texture_bind_group_layout(),
			entries: &[
				BindGroupEntry {
					binding: 0, //
					resource: BindingResource::TextureView(&texture.view),
				},
				BindGroupEntry {
					binding: 1,
					resource: BindingResource::Sampler(&texture.sampler),
				},
				BindGroupEntry {
					binding: 2, //
					resource: BindingResource::TextureView(&lightmap.view),
				},
				BindGroupEntry {
					binding: 3,
					resource: BindingResource::Sampler(&lightmap.sampler),
				},
				BindGroupEntry {
					binding: 4, //
					resource: BindingResource::TextureView(&normalmap.view),
				},
				BindGroupEntry {
					binding: 5,
					resource: BindingResource::Sampler(&normalmap.sampler),
				},
				BindGroupEntry {
					binding: 6, //
					resource: BindingResource::TextureView(&direct.view),
				},
				BindGroupEntry {
					binding: 7,
					resource: BindingResource::Sampler(&direct.sampler),
				},
			],
			label: Some(file!()),
		})
	}
}
