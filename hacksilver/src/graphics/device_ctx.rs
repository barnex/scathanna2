use super::internal::*;

/// Convenience wrapper around a configured wgpu Device.
///
/// Exposes higher-level methods to create textures, buffers, shaders, etc,
/// using a single queue (i.e. synchronously).
///
/// Includes performance counters.
pub struct DeviceCtx {
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub surface_format: wgpu::TextureFormat,
	pub counters: Counters,
}

#[derive(Copy, Clone)]
pub struct TextureOpts {
	pub max_filter: wgpu::FilterMode,
	pub format: wgpu::TextureFormat,
	pub address_mode: wgpu::AddressMode,
}

//pub const LINEAR: wgpu::FilterMode = wgpu::FilterMode::Linear;
//pub const NEAREST: wgpu::FilterMode = wgpu::FilterMode::Nearest;
//pub const SRGBA: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
//pub const LINEAR: TextureOpts = TextureOpts{max_filter: wgpu::FilterMode::Linear, ..default()};

pub const NEAREST: TextureOpts = TextureOpts {
	max_filter: wgpu::FilterMode::Nearest,
	..TextureOpts::DEFAULT
};

pub const RGBA_LINEAR: TextureOpts = TextureOpts {
	format: wgpu::TextureFormat::Rgba8Unorm,
	..TextureOpts::DEFAULT
};

pub const CLAMP_TO_EDGE: TextureOpts = TextureOpts {
	address_mode: wgpu::AddressMode::ClampToEdge,
	..TextureOpts::DEFAULT
};

//pub const RGBA_LINEAR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

impl TextureOpts {
	const DEFAULT: Self = Self {
		max_filter: wgpu::FilterMode::Linear,
		format: wgpu::TextureFormat::Rgba8UnormSrgb,
		address_mode: wgpu::AddressMode::Repeat,
	};

	pub fn filter(self, filter: wgpu::FilterMode) -> Self {
		self.with(|s| s.max_filter = filter)
	}
	pub fn format(self, format: wgpu::TextureFormat) -> Self {
		self.with(|s| s.format = format)
	}
	pub fn address_mode(self, mode: wgpu::AddressMode) -> Self {
		self.with(|s| s.address_mode = mode)
	}
}

//impl From<&GraphicsOpts> for TextureOpts {
//	fn from(opts: &GraphicsOpts) -> Self {
//		TextureOpts {
//			filter: opts.filter(),
//			format: SRGBA,
//			address_mode: wgpu::AddressMode::Repeat,
//		}
//	}
//}

//impl From<&GraphicsOpts> for TextureOpts{
//    fn from(opts: &GraphicsOpts) -> Self {
//        todo!()
//    }
//}

impl Default for TextureOpts {
	fn default() -> Self {
		Self::DEFAULT
	}
}

impl DeviceCtx {
	pub fn new(device: wgpu::Device, queue: wgpu::Queue, surface_format: TextureFormat) -> Self {
		Self {
			device,
			queue,
			surface_format,
			counters: default(),
		}
	}

	pub fn upload_buffer<T: bytemuck::Pod>(&self, dst: &wgpu::Buffer, src: &[T]) {
		self.counters.buffer_uploads.inc();
		self.counters.bytes_uploaded.add((src.len() * mem::size_of::<T>()) as u64);
		self.queue.write_buffer(dst, 0, bytemuck::cast_slice(src));
	}

	pub fn create_rgba_mipmap(&self, opts: &GraphicsOpts, mips: &[&[u8]], dimensions: uvec2, sampling: &TextureOpts) -> Texture {
		self.counters.texture_uploads.inc();

		debug_assert!(mips[0].len() == 4 * dimensions.reduce(u32::mul) as usize);
		if mips.len() > 1 {
			assert!(dimensions.x().is_power_of_two());
			assert!(dimensions.y().is_power_of_two());
		}

		let mut size = wgpu::Extent3d {
			width: dimensions.x(),
			height: dimensions.y(),
			depth_or_array_layers: 1,
		};
		let texture = self.device.create_texture(&wgpu::TextureDescriptor {
			label: Some(file!()),
			size,
			mip_level_count: mips.len() as u32,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: sampling.format,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
		});

		for (i, rgba) in mips.iter().enumerate() {
			let i = i as u32;
			self.queue.write_texture(
				wgpu::ImageCopyTexture {
					aspect: wgpu::TextureAspect::All,
					texture: &texture,
					mip_level: i,
					origin: wgpu::Origin3d::ZERO,
				},
				rgba,
				wgpu::ImageDataLayout {
					offset: 0,
					bytes_per_row: NonZeroU32::new(4 * size.width),
					rows_per_image: NonZeroU32::new(size.height),
				},
				size,
			);
			size.width /= 2;
			size.height /= 2;
		}

		let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
		let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: sampling.address_mode,
			address_mode_v: sampling.address_mode,
			address_mode_w: sampling.address_mode,
			mag_filter: sampling.max_filter,
			min_filter: sampling.max_filter,
			mipmap_filter: sampling.max_filter,
			label: Some(file!()),
			anisotropy_clamp: opts.anisotropy_clamp(),
			..default() //border_color: todo!(),
			            //lod_min_clamp: todo!(),
			            //lod_max_clamp: todo!(),
			            //compare: todo!(),
		});

		Texture { texture, view, sampler }
	}

	pub fn create_vao<T>(&self, vertices: &[T], indices: &[u32]) -> VAO
	where
		T: bytemuck::Pod,
	{
		debug_assert!(indices.len() < 1 << 31);
		self.counters.buffer_creates.inc();
		let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some(file!()),
			contents: bytemuck::cast_slice(vertices),
			usage: wgpu::BufferUsages::VERTEX,
		});
		let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some(file!()),
			contents: bytemuck::cast_slice(indices),
			usage: wgpu::BufferUsages::INDEX,
		});

		VAO {
			num_indices: indices.len() as u32,
			vertex_buffer,
			index_buffer,
		}
	}

	pub fn create_depth_texture(&self, opts: &GraphicsOpts, size: uvec2) -> Texture {
		let size = wgpu::Extent3d {
			width: size.x(),
			height: size.y(),
			depth_or_array_layers: 1,
		};
		let desc = wgpu::TextureDescriptor {
			label: Some(file!()),
			size,
			mip_level_count: 1,
			sample_count: opts.msaa_sample_count(),
			dimension: wgpu::TextureDimension::D2,
			format: Canvas::DEPTH_FORMAT,
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
		};
		let texture = self.device.create_texture(&desc);
		let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
		let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Nearest,
			compare: Some(wgpu::CompareFunction::LessEqual),
			lod_min_clamp: -100.0,
			lod_max_clamp: 100.0,
			..default()
		});

		Texture { texture, view, sampler }
	}

	pub fn create_msaa_fb(&self, opts: &GraphicsOpts, config: &wgpu::SurfaceConfiguration) -> Option<MSAAFB> {
		if opts.msaa_enabled() {
			let fb = self.create_multisampled_framebuffer(opts, config);
			let fb_view = fb.create_view(&wgpu::TextureViewDescriptor::default());
			Some(MSAAFB { fb, fb_view })
		} else {
			None
		}
	}

	pub fn create_multisampled_framebuffer(&self, opts: &GraphicsOpts, config: &wgpu::SurfaceConfiguration) -> wgpu::Texture {
		let multisampled_texture_extent = wgpu::Extent3d {
			width: config.width,
			height: config.height,
			depth_or_array_layers: 1,
		};
		let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
			size: multisampled_texture_extent,
			mip_level_count: 1,
			sample_count: opts.msaa_sample_count(),
			dimension: wgpu::TextureDimension::D2,
			format: config.format,
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			label: Some(file!()),
		};
		self.device.create_texture(multisampled_frame_descriptor)
	}
}
