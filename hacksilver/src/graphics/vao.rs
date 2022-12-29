/// High-level vertex buffer.
pub struct VAO {
	pub num_indices: u32,
	pub vertex_buffer: wgpu::Buffer,
	pub index_buffer: wgpu::Buffer,
}
