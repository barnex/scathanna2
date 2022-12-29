use super::internal::*;

type ID = usize;
type UV = vec2;
type Normal = vec3;

impl Intersect for (Face, ID) {
	type Attrib = (Normal, UV, ID);

	fn intersect(&self, r: &Ray<f32>, h: &mut HitRecord<f32, Self::Attrib>) -> bool {
		let mut h2 = HitRecord::new();
		let hit = self.0.intersect(r, &mut h2);
		if hit {
			let (normal, uv, _mat) = h2.attrib.unwrap(/*attrib always populated on hit*/);
			h.record(h2.t, &(normal, uv, self.1))
		}
		hit
	}
}

impl Bounded for (Face, ID) {
	fn bounds(&self) -> BoundingBox<f32> {
		self.0.bounds()
	}
}
