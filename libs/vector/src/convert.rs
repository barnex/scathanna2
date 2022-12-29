use super::*;

/// The `Convert` trait is similar to the standard `From` and `Into` traits,
/// but allows for a potentially lossy conversion.
pub trait Convert<U> {
	fn convert(self) -> U;
}

impl<T> Vector2<T>
where
	T: Copy,
{
	#[inline]
	pub fn convert<U>(self) -> Vector2<U>
	where
		T: Convert<U>,
	{
		self.map(T::convert)
	}
}

impl<T> Vector3<T>
where
	T: Copy,
{
	#[inline]
	pub fn convert<U>(self) -> Vector3<U>
	where
		T: Convert<U>,
	{
		self.map(T::convert)
	}
}

impl<T> Vector4<T>
where
	T: Copy,
{
	#[inline]
	pub fn convert<U>(self) -> Vector4<U>
	where
		T: Convert<U>,
	{
		self.map(T::convert)
	}
}

impl Convert<f32> for f64 {
	fn convert(self) -> f32 {
		self as f32
	}
}

impl Convert<f32> for i32 {
	fn convert(self) -> f32 {
		self as f32
	}
}

impl Convert<i32> for f64 {
	fn convert(self) -> i32 {
		self as i32
	}
}

impl Convert<f64> for f32 {
	fn convert(self) -> f64 {
		self.into()
	}
}

impl Convert<i32> for f32 {
	fn convert(self) -> i32 {
		self as i32
	}
}

impl Convert<i32> for u32 {
	fn convert(self) -> i32 {
		self as i32
	}
}

impl Convert<u8> for u32 {
	fn convert(self) -> u8 {
		self as u8
	}
}

impl Convert<f64> for i32 {
	fn convert(self) -> f64 {
		self.into()
	}
}

impl Convert<i8> for i32 {
	fn convert(self) -> i8 {
		self as i8
	}
}

impl Convert<f64> for u8 {
	fn convert(self) -> f64 {
		self.into()
	}
}

impl Convert<i32> for u8 {
	fn convert(self) -> i32 {
		self.into()
	}
}

impl Convert<f32> for u8 {
	fn convert(self) -> f32 {
		self.into()
	}
}

impl Convert<u32> for u8 {
	fn convert(self) -> u32 {
		self.into()
	}
}

impl Convert<f32> for i8 {
	fn convert(self) -> f32 {
		self.into()
	}
}

impl Convert<i32> for i8 {
	fn convert(self) -> i32 {
		self.into()
	}
}
