use super::internal::*;

pub struct Accumulator<T> {
	sum: T,
	n: usize,
}

impl<T> Accumulator<T>
where
	T: Add<Output = T> + Default + Div<f32, Output = T> + Clone,
{
	pub fn new() -> Self {
		Self { sum: T::default(), n: 0 }
	}

	pub fn add(&mut self, v: T) {
		self.sum = self.sum.clone() + v;
		self.n += 1;
	}

	pub fn avg(&self) -> Option<T> {
		if self.n == 0 {
			None
		} else {
			Some(self.sum.clone() / (self.n as f32))
		}
	}
}
