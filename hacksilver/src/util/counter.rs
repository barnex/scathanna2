use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

#[derive(Default)]
pub struct Counter(AtomicU64);

impl Counter {
	pub const fn new() -> Self {
		Self(AtomicU64::new(0))
	}

	pub fn inc(&self) -> u64 {
		self.add(1)
	}

	pub fn add(&self, rhs: u64) -> u64 {
		self.0.fetch_add(rhs, Ordering::SeqCst)
	}

	pub fn take(&self) -> u64 {
		self.0.swap(0, Ordering::SeqCst)
	}

	pub fn reset(&self) {
		self.take();
	}
}
