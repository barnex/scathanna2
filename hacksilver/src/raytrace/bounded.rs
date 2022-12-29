use super::internal::*;

pub trait IBounded {
	fn ibounds(&self) -> BoundingBox<i32>;
}

pub trait Bounded {
	fn bounds(&self) -> BoundingBox<f32>;
}

impl<T> Bounded for T
where
	T: IBounded,
{
	fn bounds(&self) -> BoundingBox<f32> {
		self.ibounds().convert()
	}
}
