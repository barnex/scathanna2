use super::internal::*;

/// Trait for anything that knows how to intersect with a `Ray`.
/// Implemented by faces, BVH trees,...
///
/// The associated type `Attrib` returns some attribute (metadata) about the intersection point.
/// E.g.: UV coordinates of the intersection point,
/// ID of the face intersected with, material at the intersection point,...
pub trait Intersect {
	type Attrib: Clone;
	fn intersect(&self, r: &Ray<f32>, h: &mut HitRecord<f32, Self::Attrib>) -> bool;

	fn intersection(&self, r: &Ray<f32>) -> HitRecord<f32, Self::Attrib> {
		let mut hr = HitRecord::new();
		self.intersect(r, &mut hr);
		hr
	}

	// TODO: optimize for Face and Node
	fn intersects(&self, r: &Ray<f32>) -> bool {
		self.intersection(r).attrib.is_some()
	}
}
