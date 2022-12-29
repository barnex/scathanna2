/*
use super::internal::*;
use std::thread::JoinHandle;

pub struct Future<T> {
	join: JoinHandle<Option<T>>,
	cancel: Cancel,
}

pub enum PollState<T>{
	Canceled,
	Pending,
	Ready(T),
}

pub fn start<T, F>(f: F) -> Future<T>
where
	T: Send + 'static,
	F: FnOnce(Cancel) -> Option<T> + Send + 'static,
{
	let cancel = Cancel::new();
	let join = {
		let cancel = cancel.clone();
		thread::spawn(move || f(cancel))
	};
	Future { join, cancel }
}

impl<T> Future<T>{
	pub fn cancel(&self){
		self.cancel.cancel()
	}
	pub fn poll(&self) -> PollState<T>{
		if self.cancel.is_canceled(){
			return PollState::Canceled
		}
		if !self.join.is_finished(){
			return PollState::Pending
		}
		self.join.join()

	}
}
*/
