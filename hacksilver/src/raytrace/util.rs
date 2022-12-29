/*
/// For to hit records (pairs of intersection distance `t` and some payload `v`),
/// return the frontmost, if any.
pub fn frontmost<T, V>(a: Option<(T, V)>, b: Option<(T, V)>) -> Option<(T, V)>
where
	V: PartialOrd,
{
	match (a, b) {
		(None, None) => None,
		(None, Some(hit)) => Some(hit),
		(Some(hit), None) => Some(hit),
		(Some(a), Some(b)) => {
			if a.1 < b.1 {
				Some(a)
			} else {
				Some(b)
			}
		}
	}
}
*/

/// Power of `base` (e.g. power of 2) nearest to `n`.
/// Used for building complete trees.
pub fn nearest_pow(n: u32, base: u32) -> u32 {
	let n = n as f64;
	let down = base.pow(f64::log(n, base as f64).floor() as u32);
	let up = base.pow(f64::log(n, base as f64).ceil() as u32);

	if (n - down as f64).abs() < (n - up as f64).abs() {
		down
	} else {
		up
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_nearest_pow2() {
		assert_eq!(nearest_pow(1, 2), 1);
		assert_eq!(nearest_pow(2, 2), 2);
		//assert_eq!(nearest_pow(3, 2), ?);
		assert_eq!(nearest_pow(4, 2), 4);
		assert_eq!(nearest_pow(5, 2), 4);
		//assert_eq!(nearest_pow(6, 2), ?);
		assert_eq!(nearest_pow(7, 2), 8);
		assert_eq!(nearest_pow(8, 2), 8);
		assert_eq!(nearest_pow(9, 2), 8);
		assert_eq!(nearest_pow(15, 2), 16);
		assert_eq!(nearest_pow(16, 2), 16);
		assert_eq!(nearest_pow(17, 2), 16);
	}

	#[test]
	fn test_nearest_pow4() {
		assert_eq!(nearest_pow(1, 2), 1);
		//assert_eq!(nearest_pow(2, 2), ?);
		assert_eq!(nearest_pow(3, 4), 4);
		assert_eq!(nearest_pow(4, 4), 4);
		assert_eq!(nearest_pow(5, 4), 4);
		assert_eq!(nearest_pow(6, 4), 4);
		assert_eq!(nearest_pow(7, 4), 4);
		assert_eq!(nearest_pow(9, 4), 4);
		assert_eq!(nearest_pow(12, 4), 16);
		assert_eq!(nearest_pow(15, 4), 16);
		assert_eq!(nearest_pow(16, 4), 16);
		assert_eq!(nearest_pow(17, 4), 16);
		assert_eq!(nearest_pow(40, 4), 64);
		assert_eq!(nearest_pow(64, 4), 64);
		assert_eq!(nearest_pow(65, 4), 64);
	}
}
