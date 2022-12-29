use super::internal::*;

// Shaders are not intended to be manually constructed.
// Use `ShaderPack` instead.
#[derive(Clone)]
pub enum Shader {
	Flat(Arc<wgpu::BindGroup>),
	Lines(Arc<wgpu::BindGroup>),
	Lightmap(Arc<wgpu::BindGroup>),
	Normalmap(Arc<wgpu::BindGroup>),
	Text(Arc<wgpu::BindGroup>),
	Editor(Arc<wgpu::BindGroup>),
	Highlight(Arc<wgpu::BindGroup>),
	Entity(Arc<wgpu::BindGroup>, mat4),
	Particles(Arc<wgpu::BindGroup>, mat4, f32),
	Animation(Arc<wgpu::BindGroup>, mat4, f32),
}
