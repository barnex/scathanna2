use super::internal::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct MatID(pub u8);

impl Into<usize> for MatID {
	fn into(self) -> usize {
		self.0 as usize
	}
}

impl From<usize> for MatID {
	fn from(id: usize) -> Self {
		Self(id as u8)
	}
}
