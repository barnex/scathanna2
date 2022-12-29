use super::internal::*;

pub struct SceneGraph {
	pub viewport: uvec2,
	pub bg_color: vec3,
	pub sun_dir: vec3,
	pub sun_color: vec3,
	pub camera: Camera,
	pub objects: Vec<Object>,
}

impl SceneGraph {
	pub fn new(viewport: uvec2) -> Self {
		Self {
			viewport,
			bg_color: vec3(1.0, 1.0, 1.0),
			sun_color: vec3::ONES,
			sun_dir: vec3(0.0, -1.0, 0.0),
			camera: default(),
			objects: default(),
		}
	}

	pub fn push(&mut self, obj: Object) {
		self.objects.push(obj)
	}
}
