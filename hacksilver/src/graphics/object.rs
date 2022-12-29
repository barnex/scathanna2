use super::internal::*;

/// TODO: rename DrawCmd
#[derive(Clone)]
pub struct Object {
	pub vao: Arc<VAO>,
	pub shader: Shader,
	pub index_range: Option<Range<u32>>,
}

impl Object {
	pub fn new(vao: &Arc<VAO>, shader: Shader) -> Self {
		Self {
			vao: vao.clone(),
			shader,
			index_range: None,
		}
	}

	pub fn vao(&self) -> &Arc<VAO> {
		&self.vao
	}

	pub fn mat(&self) -> &Shader {
		&self.shader
	}
}
