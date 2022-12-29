use super::internal::*;

/// A game map (serializable data + acceleration structures for physics).
pub struct Map {
	name: String,
	data: MapData,
	face_tree: Node<Face>,
	block_tree: Node<Block>,
}

impl Map {
	/// Load a Map from directory `assets/maps/map_name.hx`.
	pub fn load(assets: &AssetsDir, map_name: &str) -> Result<Self> {
		let data = MapData::load(&assets.map_dir(map_name))?;
		let face_tree = Self::face_tree(&data);
		let block_tree = Self::block_tree(&data);
		Ok(Self {
			name: map_name.into(),
			data,
			face_tree,
			block_tree,
		})
	}

	pub fn data(&self) -> &MapData {
		&self.data
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	fn face_tree(map_data: &MapData) -> Node<Face> {
		let faces = map_data.blocks().map(|b| b.faces()).flatten().collect::<Vec<_>>();
		let faces = optimize_faces(faces);
		Node::build_tree(faces)
	}

	fn block_tree(map_data: &MapData) -> Node<Block> {
		Node::build_tree(map_data.blocks().collect())
	}

	/// Where does a ray intersect the map, if any.
	pub fn intersect_t(&self, ray: &Ray64) -> Option<f64> {
		// TODO: decide on precision
		let ray = Ray::new(ray.start.to_f32(), ray.dir.to_f32());
		self.face_tree.intersection(&ray).maybe_t().map(|t| t as f64)
	}

	pub fn intersect(&self, ray: &Ray32) -> HitRecord<f32, (Vector3<f32>, Vector2<f32>, MatID)> {
		// TODO: decide on precision
		self.face_tree.intersection(&ray)
	}

	pub fn bumps(&self, bounds: &BoundingBox<f32>) -> bool {
		// *****************************
		// TODO: don' truncate to int!!
		// *****************************
		let imin = bounds.min.map(f32::floor).floor();
		let imax = bounds.max.map(f32::ceil).floor();

		for iz in imin.z()..=imax.z() {
			for iy in imin.y()..=imax.y() {
				for ix in imin.x()..=imax.x() {
					let pos = ivec3(ix, iy, iz);
					// TODO: more precise
					if self.block_tree.contains(pos.to_f32()) {
						return true;
					}
				}
			}
		}
		false
	}
}
