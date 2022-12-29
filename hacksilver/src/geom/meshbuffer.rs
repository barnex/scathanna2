use super::internal::*;

// TODO: generic
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct MeshBuffer {
	pub vertices: Vec<VertexLM>,
	pub indices: Vec<u32>,
}

impl MeshBuffer {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn collect<'a>(shards: impl IntoIterator<Item = &'a MeshBuffer>) -> Self {
		let mut buf = Self::new();
		for shard in shards {
			buf.append(shard)
		}
		buf
	}

	pub fn line(start: vec3, end: vec3) -> Self {
		let start = VertexLM { position: start, ..default() };
		let end = VertexLM { position: end, ..default() };

		Self {
			vertices: vec![start, end],
			indices: vec![0, 1],
		}
	}

	pub fn rect(vertices: &[VertexLM; 4]) -> Self {
		Self {
			vertices: vertices.iter().copied().collect(),
			indices: vec![0, 1, 2, 0, 2, 3],
		}
	}

	pub fn triangle(vertices: &[VertexLM; 3]) -> Self {
		Self {
			vertices: vertices.iter().copied().collect(),
			indices: vec![0, 1, 2],
		}
	}

	pub fn rect_from_vec(vertices: Vec<VertexLM>) -> Self {
		assert!(vertices.len() == 4);
		Self {
			vertices,
			indices: vec![0, 1, 2, 0, 2, 3],
		}
	}

	pub fn vertices(&self) -> &[VertexLM] {
		&self.vertices
	}

	pub fn indices(&self) -> &[u32] {
		&self.indices
	}

	/// Add a single vertex, assign it to the next free index.
	/// Vertices are typically pushed per 3.
	pub fn push(&mut self, v: VertexLM) {
		let index = self.vertices.len() as u32;
		self.vertices.push(v);
		self.indices.push(index);
	}

	pub fn append(&mut self, rhs: &MeshBuffer) {
		let offset = self.vertices.len() as u32;
		self.indices.extend(rhs.indices.iter().map(|v| v + offset));
		self.vertices.extend_from_slice(&rhs.vertices);
	}

	#[must_use = "Does not modify the original"]
	pub fn translated(&self, delta: vec3) -> Self {
		self.map_positions(|p| p + delta)
	}

	pub fn transform(&mut self, transf: &mat4) {
		for v in &mut self.vertices {
			v.transform(transf)
		}
	}

	/// A copy of `self`, with a function applied to the vertex positions.
	/// TODO: transform normals, etc.
	#[must_use = "Does not modify the original"]
	pub fn map_positions<F>(&self, f: F) -> Self
	where
		F: Fn(vec3) -> vec3,
	{
		Self {
			indices: self.indices.clone(),
			vertices: self.vertices.iter().map(|v| v.map_position(&f)).collect(),
		}
	}
}
