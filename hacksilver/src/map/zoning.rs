use super::internal::*;

pub const LOG_ZONE_SIZE: u32 = 7; // TODO: make sure zone size >= max block size.
pub const ZONE_SIZE: u32 = 1 << LOG_ZONE_SIZE;
pub const ZONE_ISIZE: i32 = ZONE_SIZE as i32;
pub const ZONE_MASK: i32 = !((1 << (LOG_ZONE_SIZE)) - 1);

/// Collect blocks in one Vec per zone, keyed by the zone position. E.g.:
///
///    +--------+--------+
///    |      c |   e    |    =>   { (32,64):[a,b,c],  (64,64):[d,e]}
///    |  a     | d      |
///    |    b   |        |
///    +--------+--------+
///  (32, 64)  (64, 64)
///
pub fn group_by_zone<'a>(blocks: impl IntoIterator<Item = &'a Block> + 'a) -> HashMap<ivec3, Vec<Block>> {
	let mut by_zone = HashMap::<ivec3, Vec<Block>>::default();
	for b in blocks {
		let key = zone_for(b);
		by_zone.entry(key).or_default().push(b.clone())
	}
	by_zone
}

/// Truncate a position to the zone it's in.
/// I.e. round all coordinates down to a multiple of the zone size.
/// E.g.:
///
///  +--------+
///  |        |
///  |  p     |
///  |↙       |
///  +--------+
/// (32, 64)   (64, 64)
///
pub fn trunc_to_zone(p: ivec3) -> ivec3 {
	p.map(|v| v & ZONE_MASK)
}

pub fn zone_for<B>(b: &B) -> ivec3
where
	B: IBounded,
{
	trunc_to_zone(b.ibounds().min)
}

pub fn is_zone_aligned(p: ivec3) -> bool {
	p == trunc_to_zone(p)
}

/// Partition values by a property.
/// Values with the same property are grouped together in the same Vec.
///
/// E.g.:
///
///   let people_by_age: HashMap<Age, Vec<Person>> = group_by(people, |p|p.age);
///
pub fn group_by<I, F, K, V>(values: I, property: F) -> HashMap<K, Vec<V>>
where
	I: IntoIterator<Item = V>,
	F: Fn(&V) -> K,
	K: std::hash::Hash + Eq,
{
	let mut grouped = HashMap::<K, Vec<V>>::default();
	for value in values.into_iter() {
		grouped.entry(property(&value)).or_default().push(value);
	}
	grouped
}
