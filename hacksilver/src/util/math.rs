use super::internal::*;
use std::ops::Mul;

pub use std::f32::consts::PI;
#[allow(unused)]
pub const DEG: f32 = PI / 180.0;

#[allow(unused)]
pub const INF: f32 = f32::INFINITY;

/*
/// Sample a random unit vector with isotropic direction.
pub fn sample_isotropic_direction(rng: &mut impl rand::Rng) -> vec3 {
	let norm = rand_distr::StandardNormal;
	vec3(rng.sample(norm), rng.sample(norm), rng.sample(norm)).normalized()
}
*/

pub fn square<T: Mul + Copy>(x: T) -> T::Output {
	x * x
}

/// Linear interpolation ('mix') between `v0` and `v1`.
/// ```
/// # use vector::*;
/// # use hacksilver::util::*;
/// let a = 1.0;
/// let b = 2.0;
/// assert_eq!(lerp(a, b, 0.2), 1.2);
///
/// let a = vec2(1.0, 2.0);
/// let b = vec2(2.0, 4.0);
/// assert_eq!(lerp(a, b, 0.5), vec2(1.5, 3.0));
/// ```
#[inline]
pub fn lerp<V, T>(v0: V, v1: V, t: T) -> V
where
	V: Add<Output = V> + Sub<Output = V> + Mul<T, Output = V> + Clone,
{
	v0.clone() + (v1 - v0) * t
}

/// Linear interpolation between points (x0, y0) and (x1, y1).
///         ^
///      y1 |-----------*
///  result |-------*   |
///      y0 |---*   |   |
///         |   |   |   |
///         +---+---+---+--->
///            x0   x  x1
#[inline]
pub fn linterp<X, Y>(x0: X, y0: Y, x1: X, y1: Y, x: X) -> Y
where
	X: Add<Output = X> + Sub<Output = X> + Div<X, Output = X> + Clone,
	Y: Add<Output = Y> + Sub<Output = Y> + Mul<Y, Output = Y> + Mul<X, Output = Y> + Clone,
{
	let t = (x - x0.clone()) / (x1 - x0);
	y0.clone() + (y1 - y0) * t
}

/*
/// Clamp a value to lie between min and max (inclusive).
/// TODO: remove, use std::ord::clamp :)
#[inline]
pub fn clamp<T>(v: T, min: T, max: T) -> T
where
	T: Copy + PartialOrd,
{
	debug_assert!(max >= min);
	if v < min {
		return min;
	}
	if v > max {
		return max;
	}
	v
}
*/

// /// Clamp a value to lie in range.
// #[inline]
// pub fn clamp_range<T>(v: T, range: Range<T>) -> T
// where
// 	T: Copy + PartialOrd,
// {
// 	if v < range.start {
// 		return range.start;
// 	}
// 	if v > range.end {
// 		return range.end;
// 	}
// 	v
// }

//pub fn zeros<T: Default>(n: usize) -> Vec<T> {
//	let mut dst = Vec::with_capacity(n);
//	for _i in 0..n {
//		dst.push(T::default());
//	}
//	dst
//}

/// Wrap an angle (in radians) to an equivalent angle in the range -PI..PI.
pub fn wrap_angle(angle: f32) -> f32 {
	if angle > PI {
		return angle - 2.0 * PI;
	}
	if angle < -PI {
		return angle + 2.0 * PI;
	}
	angle
}


#[inline]
pub fn and(a: bool, b: bool) -> bool {
	a && b
}

pub fn is_aligned_to(v: ivec3, align: u32) -> bool {
	let align = align as i32;
	v.x() % align == 0 && v.y() % align == 0 && v.z() % align == 0
}

#[cfg(test)]
mod test {

	use super::*;

	#[test]
	fn test_is_aligned_to() {
		assert_eq!(is_aligned_to(ivec3(-1, 0, 1), 1), true);
		assert_eq!(is_aligned_to(ivec3(-1, 0, 1), 2), false);
		assert_eq!(is_aligned_to(ivec3(-1, 0, 1), 4), false);
		assert_eq!(is_aligned_to(ivec3(-33, 24, 789), 1), true);
		assert_eq!(is_aligned_to(ivec3(-33, 24, 789), 2), false);
		assert_eq!(is_aligned_to(ivec3(-4, 0, 8), 2), true);
		assert_eq!(is_aligned_to(ivec3(-4, 0, 8), 4), true);
		assert_eq!(is_aligned_to(ivec3(-4, 0, 8), 8), false);
		assert_eq!(is_aligned_to(ivec3(4, 4, 4), 8), false);
		assert_eq!(is_aligned_to(ivec3(-4, 0, 8), 16), false);
		assert_eq!(is_aligned_to(ivec3(-40, 100, 80), 4), true);
		assert_eq!(is_aligned_to(ivec3(-40, 100, 80), 8), false);
		assert_eq!(is_aligned_to(ivec3(-4, 8, 4), 4), true);
	}
}
