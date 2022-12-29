use smallvec::SmallVec;

use super::internal::*;

#[derive(Default, PartialEq, Debug)]
pub struct ObjSet {
	pub mtllib: Option<String>,
	pub objects: Vec<Object>,
}

impl ObjSet {
	pub fn iter_vertex_positions(&self) -> impl Iterator<Item = vec3> + '_ {
		self.objects.iter().map(Object::vertex_positions).flatten()
	}

	pub fn iter_texture_coordinates(&self) -> impl Iterator<Item = vec2> + '_ {
		self.objects.iter().map(Object::texture_cordinates).flatten()
	}

	pub fn iter_vertex_normals(&self) -> impl Iterator<Item = vec3> + '_ {
		self.objects.iter().map(Object::vertex_normals).flatten()
	}
}

#[derive(Default, PartialEq, Debug)]
pub struct Object {
	pub name: String,
	pub mtl: Option<String>,
	pub faces: Vec<Face>,
}

impl Object {
	pub fn vertex_positions(&self) -> impl Iterator<Item = vec3> + '_ {
		self.faces.iter().flatten().map(|v| v.position)
	}

	pub fn texture_cordinates(&self) -> impl Iterator<Item = vec2> + '_ {
		self.faces.iter().flatten().map(|v| v.texture)
	}

	pub fn vertex_normals(&self) -> impl Iterator<Item = vec3> + '_ {
		self.faces.iter().flatten().map(|v| v.normal)
	}
}

pub type Face = SmallVec<[Vertex; 3]>;

#[derive(PartialEq, Debug, Clone)]
pub struct Vertex {
	pub position: vec3,
	pub texture: vec2,
	pub normal: vec3,
}
