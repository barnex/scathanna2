use super::internal::*;

#[derive(Debug)]
pub struct HitRecord<T, A>
where
	T: Float,
	A: Clone, // The type of the attribute. Cloned only when an intersection is in front of the previous HitRecord.
{
	pub t: T,              // Intersection distance (f32 or f64). Starts off with infinity.
	pub attrib: Option<A>, // Data corresponding to the hit (if any). E.g.: normal vector, material,...
}

impl<T, A> HitRecord<T, A>
where
	T: Float,
	A: Clone,
{
	#[inline]
	pub fn new() -> Self {
		Self { t: T::INF, attrib: None }
	}

	#[inline]
	pub fn maybe_t(&self) -> Option<T> {
		match self.attrib {
			None => None,
			Some(_) => Some(self.t),
		}
	}

	#[inline]
	pub fn record(&mut self, t: T, attrib: &A) {
		if t < self.t {
			self.t = t;
			self.attrib = Some(attrib.clone()); // Clone only when it will be used.
		}
	}
}
