use super::internal::*;

#[derive(Debug)]
pub struct Plane {
	pub origin: vec3,
	pub normal: vec3,
}

impl Plane {
	pub fn intersect(&self, r: &Ray<f64>) -> Option<f64> {
		let n = self.normal.to_f64();
		let s = r.start - self.origin.to_f64();
		let t = -n.dot(s) / n.dot(r.dir);

		if t < 0.0 || !t.is_finite() {
			None
		} else {
			Some(t)
		}
	}
}
