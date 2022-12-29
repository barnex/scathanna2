use super::internal::*;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Palette {
	mapping: HashMap<MatID, String>,
}

impl Palette {
	pub fn material_name_for(&self, mat: MatID) -> Option<&str> {
		self.mapping.get(&mat).map(|s| s.as_str())
	}

	pub fn set(&mut self, mat: MatID, name: &str) {
		self.mapping.insert(mat, name.to_owned());
	}

	pub fn material_for(&self, materials: &MaterialPack, mat: MatID) -> GMaterial {
		self //
			.material_name_for(mat)
			.map(|name| materials.get(name))
			.unwrap_or_else(|| materials.fallback())
	}

	pub fn host_material_for(&self, materials: &MaterialPack, mat: MatID) -> Option<Arc<HostMaterial>> {
		self //
			.material_name_for(mat)
			.map(|name| materials.get_host(name))
	}
}
