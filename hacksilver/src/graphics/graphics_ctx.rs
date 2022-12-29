use super::internal::*;

/// Context: explicit handle to "globals":
///
///   * device and command queue,
///   * shaders
///   * embedded font
///   * performance counters
///
/// These are, logically speaking, global singletons.
pub struct GraphicsCtx {
	pub dev: Arc<DeviceCtx>,
	pub opts: GraphicsOpts,
	pub shader_pack: ShaderPack,
	pub fallback_texture: Arc<Texture>,
}

impl GraphicsCtx {
	pub fn new(opts: GraphicsOpts, dev: &Arc<DeviceCtx>, surface_format: TextureFormat) -> Self {
		let dev = dev.clone();
		let fallback_texture = Arc::new(Self::_embedded_fallback_texture(&dev));

		let shader_pack = ShaderPack::new(&opts, &dev, surface_format);

		Self {
			dev,
			opts,
			shader_pack,
			fallback_texture,
		}
	}

	// TODO: remove
	pub fn device(&self) -> &wgpu::Device {
		&self.dev.device
	}

	// TODO: remove
	pub fn queue(&self) -> &wgpu::Queue {
		&self.dev.queue
	}

	// TODO: remove
	pub fn upload_buffer<T: bytemuck::Pod>(&self, dst: &wgpu::Buffer, src: &[T]) {
		self.dev.upload_buffer(dst, src)
	}

	pub fn upload_image_mip(&self, image: &DynamicImage, sampling: &TextureOpts) -> Texture {
		let mips = gen_mips(&self.opts, image);
		let mips = mips.iter().map(|vec| vec.as_ref()).collect::<Vec<_>>();
		self.dev.create_rgba_mipmap(&self.opts, &mips, image.dimensions().into(), sampling)
	}

	/// Upload RGBA pixel data to the GPU. Linear filtering.
	pub fn upload_image_nomip(&self, image: &DynamicImage, sampling: &TextureOpts) -> Texture {
		let dimensions = image.dimensions();
		let rgba = image.to_rgba8().into_raw();
		self.upload_rgba_mips(&[&rgba], dimensions.into(), sampling)
	}

	/// Upload RGBA pixel data to the GPU. Linear filtering.
	pub fn upload_rgba(&self, rgba: &[u8], dimensions: uvec2, sampling: &TextureOpts) -> Texture {
		self.upload_rgba_mips(&[rgba], dimensions, sampling)
	}

	/// Upload RGBA pixel data to the GPU. Linear filtering.
	pub fn upload_rgba_mips(&self, rgba_mips: &[&[u8]], dimensions: uvec2, sampling: &TextureOpts) -> Texture {
		self.dev.create_rgba_mipmap(&self.opts, rgba_mips, dimensions, sampling)
	}

	fn _embedded_fallback_texture(dev: &DeviceCtx) -> Texture {
		let image = &image::load_from_memory(include_bytes!("../../../assets/textures/fallback_texture.png")).expect("decode embedded texture");
		let opts = GraphicsOpts::default();
		dev.create_rgba_mipmap(&opts, &[&image.to_rgba8()], image.dimensions().into(), &NEAREST)
	}

	pub fn upload_meshbuffer(&self, buf: &MeshBuffer) -> VAO {
		self.dev.create_vao(buf.vertices(), buf.indices())
	}
}
