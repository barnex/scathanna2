use core::panic;
use std::str::FromStr;

use super::internal::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Block {
	pub pos: ivec3,
	pub rotation: Rotation,
	pub size: Vector3<u8>,

	pub typ: BlockTyp,
	pub mat: MatID,
}

// TODO: closed enum ?
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockTyp(pub u8);

const MAX_BLOCK_TYP: u8 = 3;

impl Into<usize> for BlockTyp {
	fn into(self) -> usize {
		self.0 as usize
	}
}

impl Block {
	pub fn faces(&self) -> Vec<Face> {
		match self.typ {
			BlockTyp(0) => unit_cube_faces(self.mat),
			BlockTyp(1) => unit_wedge_faces(self.mat),
			BlockTyp(2) => unit_tetra_faces(self.mat),
			BlockTyp(3) => unit_itetra_faces(self.mat),
			unknown => panic!("unsupported block type {:?}", unknown),
		}
		.iter()
		// Rotate, scale, translate
		.map(|face| face.map_positions(|p| Self::transform(self.pos, self.rotation, self.size, p)))
		.collect()
	}

	pub fn transform(pos: ivec3, rotation: Rotation, size: Vector3<u8>, p: ivec3) -> ivec3 {
		((rotation.rotate_internal(p)) * size.convert()) + pos
	}

	pub fn transform_f32(pos: ivec3, rotation: Rotation, size: Vector3<u8>, p: vec3) -> vec3 {
		((rotation.rotate_internal_f32(p)) * size.convert()) + pos.convert()
	}

	pub fn inverse_f32(pos: ivec3, rotation: Rotation, size: Vector3<u8>, p: vec3) -> vec3 {
		rotation.inverse().rotate_internal_f32((p - pos.convert()) / size.convert())
	}

	// TODO: remove, should act on faces
	pub fn intersect(&self, ray: &Ray<f64>) -> Option<f64> {
		self.bounds64().intersect(ray)
	}

	pub fn intersects(&self, ray: &Ray<f64>) -> bool {
		self.bounds64().intersects(ray)
	}

	pub fn bounds64(&self) -> BoundingBox<f64> {
		self.ibounds().convert()
	}

	pub fn bounds32(&self) -> BoundingBox<f32> {
		self.ibounds().convert()
	}

	pub fn center(&self) -> vec3 {
		self.bounds64().center().convert()
	}

	pub fn icenter(&self) -> ivec3 {
		self.bounds64().center().map(|v| v.floor() as i32)
	}

	pub fn set_center(&mut self, center: ivec3) {
		self.pos += center - self.icenter();
	}
}

//-------------------------------------------------------------------------------- Parse

impl FromStr for BlockTyp {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let i = s.parse()?;
		match i {
			0..=MAX_BLOCK_TYP => Ok(BlockTyp(i)),
			bad => Err(anyhow!("invalid block type: {}, should be 0..={}", bad, MAX_BLOCK_TYP)),
		}
	}
}

//-------------------------------------------------------------------------------- BVH Volume

impl Volume for Block {
	fn contains(&self, point: vec3) -> bool {
		// TODO: other shapes, more efficient?

		if !self.bounds32().contains(point) {
			return false;
		}

		// internal coordinates [0..1]
		let point = Self::inverse_f32(self.pos, self.rotation, self.size, point);

		match self.typ {
			BlockTyp(0) => true,
			BlockTyp(1) => point.x() + point.y() < 1.0,
			BlockTyp(2) => point.x() + point.y() + point.z() < 1.0,
			BlockTyp(3) => point.x() + point.y() + point.z() > 1.0,
			BlockTyp(bad) => panic!("invalid block type: {}", bad),
		}
	}
}

impl IBounded for Block {
	fn ibounds(&self) -> BoundingBox<i32> {
		BoundingBox {
			min: self.pos,
			max: self.pos + self.size.convert::<i32>(),
		}
	}
}

// Hack for BVH tree!!!
impl Default for Block {
	fn default() -> Self {
		Self {
			pos: default(),
			rotation: default(),
			size: default(),
			typ: BlockTyp(0),
			mat: default(),
		}
	}
}
