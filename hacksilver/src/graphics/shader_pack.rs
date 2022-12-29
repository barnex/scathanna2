use super::global_uniforms::*;
use super::internal::*;

pub struct ShaderPack {
	dev: Arc<DeviceCtx>,
	pub text_pipeline: TextPipeline,
	pub flat_texture_pipeline: FlatTexturePipeline,
	pub flat_lines_pipeline: HighlightPipeline,
	pub lightmap_pipeline: LightmapPipeline,
	pub normalmap_pipeline: NormalmapPipeline,
	pub editor_pipeline: EditorPipeline,
	pub highlight_pipeline: HighlightPipeline,
	pub entity_pipeline: EntityPipeline,
	pub particles_pipeline: ParticlesPipeline,
	pub animation_pipeline: AnimationPipeline,
	pub font_texture: Arc<Texture>,
}

impl ShaderPack {
	pub fn new(opts: &GraphicsOpts, dev: &Arc<DeviceCtx>, surface_format: TextureFormat) -> Self {
		let dev = dev.clone();
		let device = &dev.device;
		let font_texture = Arc::new(_embedded_font_texture(&dev));
		let camera_layout = GlobalUniforms::bind_group_layout(&device);
		let text_pipeline = TextPipeline::new(&opts, &device, surface_format);
		let flat_texture_pipeline = FlatTexturePipeline::new(&opts, &device, surface_format, &camera_layout, false /*lines*/);
		let flat_lines_pipeline = HighlightPipeline::new(&opts, &device, surface_format, &camera_layout, true /*lines*/);
		let highlight_pipeline = HighlightPipeline::new(&opts, &device, surface_format, &camera_layout, false /*lines*/);
		let lightmap_pipeline = LightmapPipeline::new(&opts, &device, surface_format, &camera_layout);
		let normalmap_pipeline = NormalmapPipeline::new(&opts, &device, surface_format, &camera_layout);
		let editor_pipeline = EditorPipeline::new(&opts, &device, surface_format, &camera_layout);
		let entity_pipeline = EntityPipeline::new(&opts, &device, surface_format, &camera_layout);
		let particles_pipeline = ParticlesPipeline::new(&opts, &device, surface_format, &camera_layout);
		let animation_pipeline = AnimationPipeline::new(&opts, &device, surface_format, &camera_layout);

		Self {
			dev,
			text_pipeline,
			lightmap_pipeline,
			normalmap_pipeline,
			editor_pipeline,
			flat_texture_pipeline,
			flat_lines_pipeline,
			highlight_pipeline,
			entity_pipeline,
			animation_pipeline,
			particles_pipeline,
			font_texture,
		}
	}

	pub fn text(&self) -> Shader {
		Shader::Text(Arc::new(self.text_pipeline.texture_bind_group(&self.dev, &self.font_texture)))
	}

	pub fn flat(&self, texture: &Texture) -> Shader {
		Shader::Flat(Arc::new(self.flat_texture_pipeline.texture_bind_group(&self.dev, texture)))
	}

	pub fn editor(&self, texture: &Texture) -> Shader {
		Shader::Editor(Arc::new(self.editor_pipeline.texture_bind_group(&self.dev, texture)))
	}

	pub fn lines(&self, texture: &Texture) -> Shader {
		Shader::Lines(Arc::new(self.flat_lines_pipeline.texture_bind_group(&self.dev, texture)))
	}

	pub fn lightmap(&self, texture: &Texture, lightmap: &Texture) -> Shader {
		Shader::Lightmap(Arc::new(self.lightmap_pipeline.texture_bind_group(&self.dev, texture, lightmap)))
	}

	pub fn normalmap(&self, texture: &Texture, lightmap: &Texture, normalmap: &Texture, direct: &Texture) -> Shader {
		Shader::Normalmap(Arc::new(self.normalmap_pipeline.texture_bind_group(&self.dev, texture, lightmap, normalmap, direct)))
	}

	pub fn highlight(&self, texture: &Texture) -> Shader {
		Shader::Highlight(Arc::new(self.highlight_pipeline.texture_bind_group(&self.dev, texture)))
	}

	pub fn entity(&self, texture: &Texture, transform: mat4) -> Shader {
		Shader::Entity(Arc::new(self.entity_pipeline.texture_bind_group(&self.dev, texture)), transform)
	}

	pub fn particles(&self, texture: &Texture, transform: mat4, phase: f32) -> Shader {
		Shader::Particles(Arc::new(self.particles_pipeline.texture_bind_group(&self.dev, texture)), transform, phase)
	}

	pub fn animation(&self, texture: &Texture, transform: mat4, t: f32) -> Shader {
		assert!(t >= 0.0); // TODO: debug_assert or warn
		assert!(t <= 1.0);
		Shader::Animation(Arc::new(self.animation_pipeline.texture_bind_group(&self.dev, texture)), transform, t)
	}
}

fn _embedded_font_texture(dev: &DeviceCtx) -> Texture {
	let image = &image::load_from_memory(include_bytes!("../../../assets/textures/font.png")).expect("decode embedded texture");
	let opts = GraphicsOpts::default();
	dev.create_rgba_mipmap(&opts, &[&image.to_rgba8()], image.dimensions().into(), &NEAREST)
}
