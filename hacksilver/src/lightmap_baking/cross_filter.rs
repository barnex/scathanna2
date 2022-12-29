use super::internal::*;

pub struct XFilter {
	mapping: Mapping,
	dist: u32,
	face_by_id: Vec<Face>,
	size_by_id: Vec<uvec2>,
}

type Mapping = HashMap<Key, SmallVec<[(usize, Vector2<u32>); 2]>>;
type Key = (ivec3, ivec3);

impl XFilter {
	// Build cross-face-filter from Faces and their lightmap sizes (inner size in pixels).
	// Face ID is implicit: position in the iterator.
	pub fn new(dist: u32, snips: impl Iterator<Item = (Face, uvec2)>) -> Self {
		let mut mapping = HashMap::default();
		let mut face_by_id = vec![];
		let mut size_by_id = vec![];
		for (id, (face, size)) in snips.enumerate() {
			Self::add_face(dist, &mut mapping, id, &face, size);
			face_by_id.push(face.clone());
			size_by_id.push(size);
		}
		Self {
			dist,
			mapping,
			face_by_id,
			size_by_id,
		}
	}

	fn add_face(dist: u32, mapping: &mut Mapping, face_id: usize, face: &Face, size: uvec2) {
		debug_assert!(size.x() >= 2 && size.y() >= 2);

		if size.x() < 2 || size.y() < 2 {
			// degenerate face should not occur in production
			// but don't crash if it does
			return;
		}

		let d = dist;
		for pix in cross(0..size.x(), 0..size.y()) {
			let (x, y) = pix;
			let (w, h) = size.into();

			if x < d || x >= w - d || y < d || y >= h - d {
				Self::add_entry(mapping, face_id, face, size, pix.into());
			}
		}
	}

	fn add_entry(mapping: &mut Mapping, face_id: usize, face: &Face, size: uvec2, pix: uvec2) {
		mapping.entry(Self::key_for(face, size, pix.convert())).or_default().push((face_id, pix))
	}

	// Find overlapping lightmap pixels from neighboring faces.
	// Used to "stitch" lightmap images at face edges for a seamless fit.
	//
	// E.g. position `a` is at the center of a lightmap pixel
	// shared by faces `1` and `2`.  Therefore,
	//     lookup(2, (0,0)) // position of `a` in face `2`
	// returns
	//     (face1, pos(2,0)), (face2, pos(0,0))
	//
	//   +---+---a---+
	//   |       |   |
	//   +   1   +   +
	//   |       | 2 |
	//   +---+---+   +
	//           |   |
	//           +---+
	fn lookup_overlapping(&self, face_id: usize, pix: uvec2) -> impl Iterator<Item = (usize, uvec2)> {
		let face = &self.face_by_id[face_id];
		let size = self.size_by_id[face_id];
		match self.mapping.get(&Self::key_for(face, size, pix.convert())) {
			None => smallvec![(face_id, pix)],
			Some(res) => res.clone(),
		}
		.into_iter()
	}

	fn lookup_out_of_bounds(&self, face_id: usize, pix: ivec2) -> Option<(usize, uvec2)> {
		let face = &self.face_by_id[face_id];
		let size = self.size_by_id[face_id];

		if Self::in_bounds(size, pix) {
			Some((face_id, pix.map(|v| v as u32)))
		} else {
			self.mapping.get(&Self::key_for(face, size, pix)).and_then(|list| {
				// There may be more than one pixel but all should point to the same value
				// (after stitching).
				// There may be none (if there's no neighboring face)
				list.get(0).map(Clone::clone)
			})
		}
	}

	fn in_bounds(size: uvec2, pix: ivec2) -> bool {
		let (w, h) = size.to_i32().into();
		let (x, y) = pix.into();
		x >= 0 && x < w && y >= 0 && y < h
	}

	fn key_for(face: &Face, size: uvec2, pix: ivec2) -> Key {
		let pos = Self::trunc(pixel_center_to_pos(face, size, pix));
		let norm = Self::trunc(face.normalized_normal());
		(pos, norm)
	}

	// truncate to fixed-point float
	fn trunc(v: vec3) -> ivec3 {
		v.map(|v| (v * 1024.0).round() as i32)
	}

	pub fn entries(&self) -> usize {
		self.mapping.len()
	}

	pub fn stitch(&self, snips: &[BorderedImg]) -> Vec<BorderedImg> {
		self.map(snips, &Self::stitch1)
	}

	fn stitch1(&self, snips: &[BorderedImg], face_id: usize) -> BorderedImg {
		let size = snips[face_id].inner_size();
		let mut dst = BorderedImg::new_with_inner_size(size);
		assert!(size == self.size_by_id[face_id]);

		for (ix, iy) in cross(0..size.x(), 0..size.y()) {
			let pix = uvec2(ix, iy);

			let mut acc = Accumulator::new();
			for (face_id, pix) in self.lookup_overlapping(face_id, pix) {
				acc.add(snips[face_id].at_inner_idx(pix))
			}
			dst.set_inner_idx(pix, acc.avg().expect("BUG: lookup must return at least one result"));
		}

		dst
	}

	pub fn blur(&self, snips: &[BorderedImg]) -> Vec<BorderedImg> {
		self.map(snips, &Self::blur1)
	}

	fn blur1(&self, snips: &[BorderedImg], face_id: usize) -> BorderedImg {
		assert!(self.dist >= 2);
		let size = snips[face_id].inner_size();
		let mut dst = BorderedImg::new_with_inner_size(size);
		assert!(size == self.size_by_id[face_id]);

		let gauss_3x3 = [
			((0, 0), 1.0 / 4.0),
			((-1, 0), 1.0 / 8.0),
			((1, 0), 1.0 / 8.0),
			((0, -1), 1.0 / 8.0),
			((0, 1), 1.0 / 8.0),
			((1, 1), 1.0 / 16.0),
			((1, -1), 1.0 / 16.0),
			((-1, 1), 1.0 / 16.0),
			((-1, -1), 1.0 / 16.0),
		];

		for (ix, iy) in cross(0..size.x(), 0..size.y()) {
			let pix = uvec2(ix, iy);

			let mut sum = vec3::ZERO;
			let mut sum_w = 0.0;
			for (delta, w) in gauss_3x3 {
				let pos = pix.to_i32() + delta.into();
				if let Some((face_id, pix)) = self.lookup_out_of_bounds(face_id, pos) {
					// TODO: also require that there was >= 1 valid sample
					// (e.g. don't blur outside of triangle bounds!)
					sum += w * snips[face_id].at_inner_idx(pix);
					sum_w += w;
				}
			}
			let avg = sum / sum_w;
			dst.set_inner_idx(pix, avg);
		}

		dst
	}

	pub fn despecle(&self, snips: &[BorderedImg]) -> Vec<BorderedImg> {
		self.map(snips, &Self::blur1)
	}

	fn despecle1(&self, snips: &[BorderedImg], face_id: usize) -> BorderedImg {
		/*
		assert!(self.dist >= 2);

		let size = snips[face_id].inner_size();
		let mut dst = BorderedImg::new_with_inner_size(size);
		assert!(size == self.size_by_id[face_id]);

		// 8 nearest neighbors (so excl. self).
		let neigh = [(-1, 0), (1, 0), (0, -1), (0, 1), (1, 1), (1, -1), (-1, 1), (-1, -1)];

		for (ix, iy) in cross(0..size.x(), 0..size.y()) {
			let pix = uvec2(ix, iy);

			let (mut min, mut max) = (INF, -INF);
			for delta in neigh {
				let pos = pix.to_i32() + delta.into();
				if let Some((face_id, pix)) = self.lookup_out_of_bounds(face_id, pos) {
					let color = snips[face_id].at_inner_idx(pix);
					if color.reduce(f32::min) < min{
						//min = 
					}
				}
			}
			let avg = sum / sum_w;
			dst.set_inner_idx(pix, avg);
		}

		dst
		*/
		todo!()
	}

	// Apply a function (e.g. blur, stitch,...) to all snippets.
	fn map<F>(&self, snips: &[BorderedImg], f: F) -> Vec<BorderedImg>
	where
		F: Fn(&Self, &[BorderedImg], usize) -> BorderedImg,
	{
		snips.iter().enumerate().map(|(id, _)| f(&self, snips, id)).collect()
	}
}

#[cfg(test)]
mod test {
	use super::*;

	//  0,0     2,0  3,0
	//   +---+---a---+
	//   |       |   |
	//   +   0   +   +
	//   |       | 1 |
	//   +---+---+   +
	//  0,2      |   |
	//           +---+
	//          2,3  3,3
	#[test]
	fn test_xfilter() {
		let cfg = BakeOpts::default().with(|cfg| cfg.lightmap_resolution = 1);
		let faces = vec![
			//
			Face::rectangle(MatID(0), (0, 0, 0), (2, 0, 0), (2, 2, 0), (0, 2, 0)),
			Face::rectangle(MatID(0), (2, 0, 0), (3, 0, 0), (3, 3, 0), (2, 3, 0)),
		];
		let sizes = faces.iter().map(|face| lightmap_size_no_margin(&cfg, face)).collect::<Vec<_>>();

		let filter = XFilter::new(1, faces.into_iter().zip(sizes.into_iter()));

		let mapping = filter.mapping.clone();
		let mut keys = mapping.keys().collect::<Vec<_>>();
		keys.sort_by_key(|key| (key.0.tuple(), key.1.tuple()));
		for key in keys {
			println!("{key:?} => {:?}", mapping[&key])
		}

		assert_eq!(filter.entries(), 13); // all edge pixels (16 total but 3 shared)
		                          //let lookup = |id, pos| filter.lookup(id, pos).collect::<Vec<(usize, uvec2)>>();
		                          //assert_eq!(lookup(0, uvec2(0, 0)), vec![(0, uvec2(0, 0))]);
	}
}
