use super::global_uniforms::*;
use super::internal::*;
use winit::window::Window;

/// A `Canvas` can be drawn on.
/// It hides a WGPU Surface, depth texture and context
/// and provides higher-level drawing functionality.
pub struct Canvas {
	ctx: Arc<GraphicsCtx>,
	surface: wgpu::Surface,
	camera_uniform_data: GlobalUniforms,
	instance_host_data: Vec<InstanceRaw>,
	instance_buffer: wgpu::Buffer,

	config: wgpu::SurfaceConfiguration,
	depth_texture: Texture,

	// multi-sampled framebuffer, populated if MSAA is enabled
	msaa_fb: Option<MSAAFB>,
}

pub struct MSAAFB {
	pub fb: wgpu::Texture,
	pub fb_view: wgpu::TextureView,
}

pub const MAX_ENTITIES: usize = 1024;

impl Canvas {
	pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

	/// A `Canvas` that will present to `window`.
	/// NOTE: `resize` must be called whenever the window is resized.
	pub fn new(opts: GraphicsOpts, window: &Window) -> Result<Self> {
		pollster::block_on(Self::new_async(opts, window))
	}

	async fn new_async(opts: GraphicsOpts, window: &Window) -> Result<Self> {
		let instance = wgpu::Instance::new(wgpu::Backends::all());
		let surface = unsafe { instance.create_surface(window) };
		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::HighPerformance,
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			})
			.await
			.ok_or(anyhow!("No graphics adapter found"))?;
		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: None,
					//features: wgpu::Features::POLYGON_MODE_LINE, // TODO: does not work on older graphics drivers but needed for editor
					features: wgpu::Features::default(),
					limits: wgpu::Limits::default(),
				},
				None, // Trace path
			)
			.await?;

		LOG.write(format!("Graphics adapter: {:?}", &adapter.get_info().name));

		let size = window.inner_size();
		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface
				.get_supported_formats(&adapter)
				.get(0 /*the preferred format*/)
				.copied()
				.ok_or(anyhow!("No graphics adapter found"))?,
			width: size.width,
			height: size.height,
			present_mode: match opts.vsync {
				true => wgpu::PresentMode::Fifo,
				false => wgpu::PresentMode::Immediate,
			},
			alpha_mode: wgpu::CompositeAlphaMode::Auto,
		};
		surface.configure(&device, &config);

		let instance_buffer = InstanceRaw::new_buffer(&device, &[InstanceRaw::default(); MAX_ENTITIES]);

		let dev = Arc::new(DeviceCtx::new(device, queue, config.format));

		let depth_texture = dev.create_depth_texture(&opts, uvec2(config.width, config.height));
		let msaa_fb = dev.create_msaa_fb(&opts, &config);

		let ctx = Arc::new(GraphicsCtx::new(opts, &dev, config.format));
		let camera_uniform_data = GlobalUniforms::new(ctx.device());

		Ok(Self {
			ctx,
			surface,
			config,
			depth_texture,
			camera_uniform_data,
			instance_host_data: vec![],
			instance_buffer,
			msaa_fb,
		})
	}

	/// Must be called whenever the corresponding Window got resized,
	/// so that the Canvas size fits.
	pub fn resize(&mut self, new_size: uvec2) {
		info!("resize: {}", new_size);
		if new_size.x() > 0 && new_size.y() > 0 {
			self.config.width = new_size.x();
			self.config.height = new_size.y();
			self.surface.configure(self.ctx.device(), &self.config);
			self.depth_texture = self.ctx.dev.create_depth_texture(&self.ctx.opts, uvec2(self.config.width, self.config.height));
			self.msaa_fb = self.ctx.dev.create_msaa_fb(&self.ctx.opts, &self.config);
		}
	}

	/// The Canvas' current size (width, height) in pixels.
	pub fn viewport_size(&self) -> uvec2 {
		uvec2(self.config.width, self.config.height)
	}

	/// The graphics context this canvas was created in.
	/// (May be freely cloned and passed around)
	pub fn graphics_context(&self) -> &Arc<GraphicsCtx> {
		&self.ctx
	}
}

// --------------------------------------------------------------------------------  rendering

impl Canvas {
	pub fn render(&mut self, scene: SceneGraph) {
		let surface_tex = match self.surface.get_current_texture() {
			Ok(v) => v,
			Err(wgpu::SurfaceError::Lost) => return self.handle_surface_lost(),
			Err(wgpu::SurfaceError::OutOfMemory) => panic!("out of memory"),
			Err(wgpu::SurfaceError::Outdated) => return, // Should be resolved by the next frame
			Err(wgpu::SurfaceError::Timeout) => return,  // Should be resolved by the next frame
		};
		self.render_to_surface(scene, &surface_tex);
		surface_tex.present();
	}

	// Copy entity transforms into staging buffer.
	fn upload_instance_buffer(&mut self, scene: &SceneGraph) {
		self.instance_host_data.clear();

		let mut push_instance_data = |data| {
			if self.instance_host_data.len() < MAX_ENTITIES {
				self.instance_host_data.push(data);
			} else {
				eprintln!("max entities reached");
			}
		};
		let mut push_transform = |model_matrix: &mat4, extra: vec2| {
			push_instance_data(InstanceRaw {
				model_matrix: model_matrix.clone().into(),
				extra: extra.into(),
			})
		};

		for obj in &scene.objects {
			match &obj.shader {
				Shader::Entity(_, transf) => push_transform(transf, default()),
				Shader::Particles(_, transf, t) => push_transform(transf, vec2(*t, 0.0)),
				Shader::Animation(_, transf, t) => push_transform(transf, vec2(*t, 0.0)),
				Shader::Flat(_) => (),
				Shader::Lines(_) => (),
				Shader::Lightmap(_) => (),
				Shader::Normalmap(_) => (),
				Shader::Text(_) => (),
				Shader::Editor(_) => (),
				Shader::Highlight(_) => (),
			}
		}

		// upload staging buffer to GPU (only the used part, we usually have far fewer than `MAX_ENTITIES` entities).
		if self.instance_host_data.len() != 0 {
			let n = self.instance_host_data.len().clamp(0, MAX_ENTITIES);
			self.ctx.upload_buffer(&self.instance_buffer, &self.instance_host_data[..n]);
		}
	}

	fn render_to_surface(&mut self, sg: SceneGraph, surface_tex: &wgpu::SurfaceTexture) {
		// upload camera uniforms (in ctx.queue)
		self.ctx
			.upload_buffer(&self.camera_uniform_data.buffer, &[GlobalsHostData::from(&sg.camera, self.viewport_size(), sg.sun_dir, sg.sun_color)]);

		// upload instance transforms for all `Entity` shaders.
		self.upload_instance_buffer(&sg);

		let counters = &self.ctx.dev.counters;
		let shaders = &self.ctx.shader_pack;

		let surface_view = surface_tex.texture.create_view(&wgpu::TextureViewDescriptor::default());
		let mut encoder = self.command_encoder();
		{
			let mut render_pass = self.begin_render_pass(&mut encoder, &surface_view, sg.bg_color);

			// TODO: re-order for minimal shader / texture switching
			let mut entity_instance_counter = 0;

			for obj in &sg.objects {
				let mut entity_instance_id: u32 = 0; // here be mut dragons. Set to counter++ by Entity shader to mirror upload_entity_buffer
				let mut advance_instance_id = || {
					entity_instance_id = entity_instance_counter;
					entity_instance_counter += 1;
				};
				use Shader::*;
				match &obj.shader {
					Text(bind_group) => {
						render_pass.set_pipeline(&shaders.text_pipeline.pipeline);
						render_pass.set_bind_group(0, bind_group, &[]);
					}
					Flat(bind_group) => {
						render_pass.set_pipeline(&shaders.flat_texture_pipeline.pipeline);
						render_pass.set_bind_group(0, bind_group, &[]);
						render_pass.set_bind_group(1, &self.camera_uniform_data.bind_group, &[]);
					}
					Lines(bind_group) => {
						render_pass.set_pipeline(&shaders.flat_lines_pipeline.pipeline);
						render_pass.set_bind_group(0, bind_group, &[]);
						render_pass.set_bind_group(1, &self.camera_uniform_data.bind_group, &[]);
					}
					Highlight(bind_group) => {
						render_pass.set_pipeline(&shaders.highlight_pipeline.pipeline);
						render_pass.set_bind_group(0, bind_group, &[]);
						render_pass.set_bind_group(1, &self.camera_uniform_data.bind_group, &[]);
					}
					Editor(bind_group) => {
						render_pass.set_pipeline(&shaders.editor_pipeline.pipeline);
						render_pass.set_bind_group(0, bind_group, &[]);
						render_pass.set_bind_group(1, &self.camera_uniform_data.bind_group, &[]);
					}
					Lightmap(bind_group) => {
						render_pass.set_pipeline(&shaders.lightmap_pipeline.pipeline);
						render_pass.set_bind_group(0, bind_group, &[]);
						render_pass.set_bind_group(1, &self.camera_uniform_data.bind_group, &[]);
					}
					Normalmap(bind_group) => {
						render_pass.set_pipeline(&shaders.normalmap_pipeline.pipeline);
						render_pass.set_bind_group(0, bind_group, &[]);
						render_pass.set_bind_group(1, &self.camera_uniform_data.bind_group, &[]);
					}
					Entity(bind_group, _) => {
						render_pass.set_pipeline(&shaders.entity_pipeline.pipeline);
						render_pass.set_bind_group(0, bind_group, &[]);
						render_pass.set_bind_group(1, &self.camera_uniform_data.bind_group, &[]);
						// ! keep the pace with the transforms copied by upload_instance_buffer().
						advance_instance_id();
					}
					Particles(bind_group, _, _) => {
						render_pass.set_pipeline(&shaders.particles_pipeline.pipeline);
						render_pass.set_bind_group(0, bind_group, &[]);
						render_pass.set_bind_group(1, &self.camera_uniform_data.bind_group, &[]);
						// ! keep the pace with the transforms copied by upload_instance_buffer().
						advance_instance_id();
					}
					Animation(bind_group, _, _) => {
						render_pass.set_pipeline(&shaders.animation_pipeline.pipeline);
						render_pass.set_bind_group(0, bind_group, &[]);
						render_pass.set_bind_group(1, &self.camera_uniform_data.bind_group, &[]);
						// ! keep the pace with the transforms copied by upload_instance_buffer().
						advance_instance_id();
					}
				}

				// TODO: handle gracefully (e.g. do not draw: `continue`). clamping will still draw, just use previous entity transform
				if entity_instance_id >= MAX_ENTITIES as u32 {
					panic!("too many entities")
				}
				//let entity_instance_id = entity_instance_id.clamp(0, MAX_ENTITIES as u32 - 1);

				/*
				 TODO: particle explosion used too many indices??
				In a RenderPass
				  note: encoder = `hacksilver/src/graphics/canvas.rs`
				In a draw command, indexed:true indirect:false
				  note: render pipeline = `<RenderPipeline-(8, 1, Vulkan)>`
				index 4984 extends beyond limit 3000. Did you bind the correct index buffer?
				*/

				let index_range = obj.index_range.clone().unwrap_or(0..obj.vao.num_indices);
				let num_indices = index_range.end - index_range.start;
				let vao_slice = obj.vao.vertex_buffer.slice(..);
				render_pass.set_vertex_buffer(0, vao_slice); // used by all shaders
				render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..)); // only used by entity.wgsl
				render_pass.set_index_buffer(obj.vao.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
				let instances = entity_instance_id..(entity_instance_id + 1);
				let num_instances = instances.clone().count() as u64;

				counters.draw_calls.inc();
				counters.draw_instances.add(num_instances);
				counters.vertices.add(num_instances * (num_indices as u64));

				render_pass.draw_indexed(index_range, 0, instances);
			}
			drop(render_pass);
		}

		self.ctx.queue().submit(std::iter::once(encoder.finish()));
	}

	fn command_encoder(&self) -> wgpu::CommandEncoder {
		self.ctx.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(file!()) })
	}

	fn begin_render_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder, color_view: &'a wgpu::TextureView, clear_color: vec3) -> wgpu::RenderPass<'a> {
		// ! switch based on MSAA
		let (view, resolve_target) = match &self.msaa_fb {
			None => (color_view, None),
			Some(MSAAFB { fb_view, .. }) => (fb_view, Some(color_view)),
		};

		encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some(file!()),
			color_attachments: &[
				//
				Some(wgpu::RenderPassColorAttachment {
					view,           // ! depends on MSAA
					resolve_target, //  ! depends on MSAA
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: clear_color.x().into(),
							g: clear_color.y().into(),
							b: clear_color.z().into(),
							a: 1.0,
						}),
						store: true,
					},
				}),
			],
			depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
				view: &self.depth_texture.view,
				depth_ops: Some(wgpu::Operations {
					load: wgpu::LoadOp::Clear(1.0),
					store: true,
				}),
				stencil_ops: None,
			}),
		})
	}

	// To be called if a winit redraw returns SurfaceError::Lost.
	fn handle_surface_lost(&mut self) {
		info!("handle_surface_lost");
		self.resize((self.config.width, self.config.height).into())
	}
}
