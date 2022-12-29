use super::internal::*;

#[derive(Default)]
pub struct Diffs(Vec<Envelope<ServerMsg>>);

impl Diffs {
	pub fn push(&mut self, msg: Envelope<ServerMsg>) {
		self.0.push(msg)
	}

	pub fn into_iter(self) -> impl Iterator<Item = Envelope<ServerMsg>> {
		self.0.into_iter()
	}
}
