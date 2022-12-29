use super::internal::*;

/// A `Ray` is a half-line defined by a starting point
/// and direction (unit vector).
/// Positions along the `Ray` are measured by their distance
/// `t` from the start:
///
///   start
///     +------|-------|-------|------->
///    t=0    t=1     t=2     t=3
///   
#[derive(Debug, Clone)]
pub struct Ray<T>
where
	T: Float,
{
	pub start: Vector3<T>,
	pub dir: Vector3<T>,
}

impl<T> Ray<T>
where
	T: Float,
{
	/// Constructs a ray with given starting point and direction.
	/// Both must be finite, and dir must be a unit vector.
	#[inline]
	pub fn new(start: Vector3<T>, dir: Vector3<T>) -> Self {
		#[cfg(debug_assertions)]
		{
			if !start.is_finite() {
				panic!("Ray::new: start not finite: {}", start)
			}
			if (dir.len() - T::ONE).as_f64().abs() > 1e-5 {
				panic!("Ray::new: dir not normalized: {:?}, len = {}", dir, dir.len());
			}
		}

		Self { start, dir }
	}

	/// The ray with its starting point offset by `delta_t` along the ray direction.
	#[must_use]
	#[inline]
	pub fn offset(&self, delta_t: T) -> Self {
		Self::new(self.start + self.dir * delta_t, self.dir)
	}

	/// Point at distance `t` (positive) from the start.
	#[inline]
	pub fn at(&self, t: T) -> Vector3<T> {
		self.start + self.dir * t
	}
}

impl<T> Ray<T>
where
	T: Float,
{
	pub fn convert<U>(&self) -> Ray<U>
	where
		T: Convert<U>,
		U: Float,
	{
		Ray::new(self.start.convert(), self.dir.convert())
	}
}

pub type Ray64 = Ray<f64>;
pub type Ray32 = Ray<f32>;
