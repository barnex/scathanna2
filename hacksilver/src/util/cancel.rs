use super::internal::*;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::*;

/// A shared atomic boolean to communicate thread cancelation.
#[derive(Default, Clone)]
pub struct Cancel {
	cancel: Arc<AtomicBool>,
}

impl Cancel {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn cancel(&self) {
		self.cancel.store(true, SeqCst)
	}

	pub fn is_canceled(&self) -> bool {
		self.cancel.load(SeqCst)
	}
}
