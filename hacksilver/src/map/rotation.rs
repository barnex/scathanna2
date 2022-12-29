use super::internal::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
// TODO: private
pub struct Rotation(Matrix3<i8>);

impl Rotation {
	pub const UNIT: Self = Self(Matrix3::UNIT);
	pub const ROTX90: Self = Self(Matrix3([
		Vector3::new(1, 0, 0), //
		Vector3::new(0, 0, -1),
		Vector3::new(0, 1, 0),
	]));
	pub const ROTY90: Self = Self(Matrix3([
		Vector3::new(0, 0, -1), //
		Vector3::new(0, 1, 0),
		Vector3::new(1, 0, 0),
	]));
	pub const ROTZ90: Self = Self(Matrix3([
		Vector3::new(0, -1, 0), //
		Vector3::new(1, 0, 0),
		Vector3::new(0, 0, 1),
	]));

	/// 90 degree rotations around x, y, z, respectively.
	pub const AROUND: [Rotation; 3] = [Self::ROTX90, Self::ROTY90, Self::ROTZ90];

	/// Apply the rotation around (0.5, 0.5, 0.5),
	/// i.e. "internal" to the unit cube.
	///
	/// Block primitives (cube, wedge,...) all have vertices
	/// with coordinates between 0 and 1. This internal rotation
	/// maps them to coordinates again between 0 and 1.
	///
	/// If `R` is the rotation matrix and `p` the point to be transformed,
	/// then we calculate:
	///
	///       R * (p - (0.5, 0.5, 0.5)) + (0.5, 0.5, 0.5)
	///     = R * p + (I - R)*(0.5, 0.5, 0.5)
	///
	/// (where `I` is the unit matrix).
	/// The 2nd form can be calculated without floating point arithmetic,
	/// as (I - R)'s components are all multiples of 2.
	pub fn rotate_internal(&self, p: ivec3) -> ivec3 {
		debug_assert!(p.iter().all(|v| v.abs() <= 1));
		let r = self.0.convert::<i32>();
		let off = (Matrix3::<i32>::UNIT - r) * ivec3::ONES;
		debug_assert!(off.iter().all(|v| v % 2 == 0));
		(r * p) + (off / 2)
	}

	pub fn rotate_internal_f32(&self, p: vec3) -> vec3 {
		let r = self.0.convert::<f32>();
		let off = (Matrix3::<f32>::UNIT - r) * vec3::ONES;
		(r * p) + (off / 2.0)
	}

	/// Rotate a size. I.e., if we rotate a bounding box of given size,
	/// what will the new position and size be?  (The position can change because the size is strictly positive).
	///
	///      +-------+   rot 90
	///      |       |   =>
	///  old *-------+        +----+
	///  pos                  |    |
	///                       |    |
	///                       |    |
	///                   new *----+
	///                   pos
	pub fn rotate_bounds(&self, size: Vector3<u8>) -> (Vector3<u8>, ivec3) {
		let old_diag = size.map(|v| v as i32);
		let new_diag = self.matrix().convert::<i32>() * old_diag;
		let new_bounds = BoundingBox::from_points([ivec3::ZERO, new_diag].into_iter()).unwrap();
		let new_size = new_bounds.size().map(|v| v as u8);
		let pos_offset = new_bounds.min;
		(new_size, pos_offset)
	}

	pub fn rotate_pos(&self, pos: ivec3) -> ivec3 {
		self.matrix().convert::<i32>() * pos
	}

	pub fn inverse(&self) -> Self {
		Self(self.0.transpose())
	}

	pub fn matrix(&self) -> Matrix3<i8> {
		self.0
	}
}

impl From<Matrix3<i8>> for Rotation {
	fn from(m: Matrix3<i8>) -> Self {
		debug_assert!(m.0.iter().map(|v| v.iter()).flatten().all(|v| v.abs() <= 1));
		Self(m)
	}
}

impl Default for Rotation {
	fn default() -> Self {
		Self::UNIT
	}
}

impl Mul for Rotation {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self {
		Self(self.0 * rhs.0)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn rotate_internal() {
		// Unit matrix does nothing
		assert_eq!(Rotation::UNIT.rotate_internal(ivec3(0, 0, 0)), ivec3(0, 0, 0));
		assert_eq!(Rotation::UNIT.rotate_internal(ivec3(1, 0, 0)), ivec3(1, 0, 0));
		assert_eq!(Rotation::UNIT.rotate_internal(ivec3(0, 1, 0)), ivec3(0, 1, 0));
		assert_eq!(Rotation::UNIT.rotate_internal(ivec3(0, 0, 1)), ivec3(0, 0, 1));
		assert_eq!(Rotation::UNIT.rotate_internal(ivec3(1, 1, 1)), ivec3(1, 1, 1));

		// Rotate around Z (CCW seen from Z, but appears CW here as Z points out of the screen):
		//
		//  (0,1)  -> (1,1)
		//      +------+
		//   ^  |      | |
		//   |  |      | v
		//      +------+
		//  (0,0) <-  (1,0)
		//
		//
		//  y ^
		//    |
		//    +---> x
		//  z (upwards)
		assert_eq!(Rotation::ROTZ90.rotate_internal(ivec3(0, 0, 0)), ivec3(0, 1, 0));
		assert_eq!(Rotation::ROTZ90.rotate_internal(ivec3(0, 1, 0)), ivec3(1, 1, 0));
		assert_eq!(Rotation::ROTZ90.rotate_internal(ivec3(1, 1, 0)), ivec3(1, 0, 0));
		assert_eq!(Rotation::ROTZ90.rotate_internal(ivec3(1, 0, 0)), ivec3(0, 0, 0));
	}
}
