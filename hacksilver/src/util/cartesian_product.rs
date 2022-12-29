/// The cross-product (Cartesian product) of two iterators.
///```
///     # use hacksilver::util::*;
/// 	assert_eq!(cross(1..4, 4..6).collect::<Vec<_>>(),
/// 		vec![(1, 4), (2, 4), (3, 4), (1, 5), (2, 5), (3, 5)]);
///```
pub fn cross<A, B, IX, IY>(x: IX, y: IY) -> impl Iterator<Item = (A, B)>
where
	A: Copy + 'static,
	B: Copy + 'static,
	IX: IntoIterator<Item = A> + Clone,
	IY: IntoIterator<Item = B>,
{
	let y = y.into_iter();
	y.flat_map(move |y| x.clone().into_iter().map(move |x| (x, y)))
}

/// The cross-product (Cartesian product) of three iterators.
///```
///     # use hacksilver::util::*;
/// 	assert_eq!(cross3(0..=1, 10..=11, 100..=101).collect::<Vec<_>>(),
/// 		vec![(0, 10, 100), (1, 10, 100), (0, 11, 100), (1, 11, 100), (0, 10, 101), (1, 10, 101), (0, 11, 101), (1, 11, 101)]);
///```
pub fn cross3<T, IX, IY, IZ>(x: IX, y: IY, z: IZ) -> impl Iterator<Item = (T, T, T)>
where
	T: Copy + 'static,
	IX: IntoIterator<Item = T> + Clone,
	IY: IntoIterator<Item = T> + Clone,
	IZ: IntoIterator<Item = T>,
{
	cross(x, cross(y, z)).map(|(x, (y, z))| (x, y, z))
}
