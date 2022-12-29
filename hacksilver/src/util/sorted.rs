/// Return vector in sorted order.
pub fn sorted<T: PartialOrd>(v: Vec<T>) -> Vec<T> {
	let mut v = v;
	v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
	v
}
