use std::cell::Cell;

/// Caches a single value.
pub struct Cache<T: Clone>(Cell<Option<T>>);

impl<T: Clone> Cache<T> {
	/// A clone of the cached value,
	/// or initialize first if needed.
	pub fn clone_or<F: FnOnce() -> T>(&self, f: F) -> T {
		let obj = self.0.take();
		let obj = match obj {
			Some(obj) => obj,
			None => f(),
		};
		let res = obj.clone();
		self.0.set(Some(obj));
		res
	}

	pub fn clear(&self) {
		self.0.set(None)
	}
}

impl<T: Clone> Default for Cache<T> {
	fn default() -> Self {
		Self(Cell::new(None))
	}
}
