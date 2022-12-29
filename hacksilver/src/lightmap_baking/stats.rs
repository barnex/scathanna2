pub use super::internal::*;

#[derive(Default, Clone)]
pub struct Stats {
	pub sum: dvec3,
	pub sum_sq: dvec3,
	pub n: usize,
}

impl Stats {
	pub fn add_sample(&mut self, color: vec3) {
		let color = color.convert::<f64>();
		self.sum += color;
		self.sum_sq += color * color;
		self.n += 1;
	}

	pub fn add(&mut self, rhs: &Stats) {
		self.sum += rhs.sum;
		self.sum_sq += rhs.sum_sq;
		self.n += rhs.n;
	}

	pub fn avg(&self) -> vec3 {
		match self.n {
			0 => vec3::ZERO,
			n => self.sum.to_f32() / (n as f32),
		}
	}

	pub fn error_squared(&self) -> f64 {
		self.var() / (self.n as f64)
	}

	pub fn stddev(&self) -> f64 {
		f64::sqrt(self.var())
	}

	pub fn var(&self) -> f64 {
		let n = self.n as f64;

		// average variances over x, y, z components
		let mut var = 0.0;
		for i in 0..3 {
			var += (self.sum_sq[i] / n) - square(self.sum[i] / n)
		}
		let var = var / 3.0;

		// if the true variance is very close to zero,
		// then round-off errors can cause it to come out negative.
		if var < 0.0 {
			0.0
		} else {
			var
		}
	}
}
