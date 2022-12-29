use super::internal::*;

pub trait Volume {
	/// Does the Volume contain a point?
	fn contains(&self, point: vec3) -> bool;
}
