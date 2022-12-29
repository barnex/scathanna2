use image::GenericImage;

use super::internal::*;

// TODO: should not be needed w/ proper mipmapping
const LIGHTMAP_MARGIN: u32 = 1;

/// Create an atlas image, place snippets on the atlas,
/// return atlas and pixel positions of each snippet
/// (positions to be passed through to shaders).
pub fn lm_alloc_and_copy_snips(snips: &[Snippet2]) -> (RgbImage, Vec<uvec2>) {
	let (atlas_size, positions) = lm_alloc_snips(snips);
	let mut atlas = RgbImage::new(atlas_size.x(), atlas_size.y());
	lm_copy_snips(&mut atlas, snips.iter().map(|snip| &snip.direct_visibility), &positions);
	(atlas, positions)
}

/// Return an atlas size that can accommodate all snippets,
/// and allocate pixel positions for each snippet in that atlas.
pub fn lm_alloc_snips(snips: &[Snippet2]) -> (uvec2, Vec<uvec2>) {
	// double atlas size until it fits
	const MIN_LOG_SIZE: u32 = 5; // 32
	const MAX_LOG_SIZE: u32 = 11; // 2048
	for n in MIN_LOG_SIZE..=MAX_LOG_SIZE {
		let n = 1 << n;
		let atlas_size = uvec2(n, n);
		let sizes = snips.iter().map(|snip| snip.dimensions()).collect::<Vec<uvec2>>();
		if let Ok(offsets) = LightmapAllocator::new(n, LIGHTMAP_MARGIN).alloc_all(&sizes) {
			return (atlas_size, offsets);
		}
	}
	panic!("lightmap larger than {} x {}", 1 << MAX_LOG_SIZE, 1 << MAX_LOG_SIZE);
}

/// Copy images to their position in an atlas.
pub fn lm_copy_snips<'imgs>(atlas: &mut RgbImage, imgs: impl Iterator<Item = &'imgs RgbImage> + 'imgs, positions: &[uvec2]) {
	for (img, uv) in imgs.zip(positions.iter()) {
		atlas.copy_from(img, uv.x(), uv.y()).expect("copy to lightmap atlas");
	}
}

struct LightmapAllocator {
	size: u32,
	/// Additional margin between islands.
	/// An absolute minimum of 1 is needed to avoid light bleeding.
	/// 1 additional texel is added to avoid minuscule light bleeding due to round-off under grazing incidence
	/// (see fn VoxelWorld::add_borders).
	margin: u32,
	curr: uvec2,
	next_y: u32,
}

impl LightmapAllocator {
	pub fn new(size: u32, margin: u32) -> Self {
		Self {
			size,
			margin,
			curr: uvec2(margin, margin),
			next_y: margin,
		}
	}

	/// Call `alloc` on each face.
	pub fn alloc_all(&mut self, sizes: &[uvec2]) -> Result<Vec<uvec2>> {
		let mut id_sizes = sizes.iter().enumerate().collect::<Vec<_>>();
		id_sizes.sort_by_key(|(_, size)| -(size.y() as i32)); // reduces lightmap fullness 3-10x
		let mut id_uvs = id_sizes //
			.into_iter()
			.map(|(i, &size)| self.alloc(size).map(|pos| (i, pos)))
			.collect::<Result<Vec<_>>>()?;
		id_uvs.sort_by_key(|(id, _)| *id);
		Ok(id_uvs.into_iter().map(|(_, uv)| uv).collect())
	}

	/// Allocate a `(W+1)x(H+1)` island for a `WxH` face.
	///
	/// texel 0   1   2   3   4   5   6...
	/// .   0 .   .   .   .   .   .   .
	/// .       +---+       +-------+
	/// .   1 . | . | .   . | .   . | .
	/// .       +---+       |       |
	/// .   2 .   .   .   . | .   . | .
	/// .                   +-------+
	/// .   3 .   .   .   .   .   .   .
	/// .
	/// .   4 .   .   .   .   .   .   .
	fn alloc(&mut self, size: uvec2) -> Result<uvec2> {
		debug_assert!(size.x() > 0 && size.y() > 0);

		let margin = self.margin;
		let size = size + uvec2(margin, margin);

		if self.curr.x() + size.x() >= self.size {
			// next line
			self.curr[0] = margin;
			self.curr[1] = self.next_y;
		}

		self.next_y = u32::max(self.next_y, self.curr.y() + size.y() + margin);

		let result = self.curr;
		self.curr[0] += size.x() + margin;

		let max = result + size;
		if max.iter().any(|v| v >= self.size) {
			Err(anyhow!("lightmap too small"))
		} else {
			Ok(result)
		}
	}

	/*
	pub fn fullness(&self) -> f32 {
		(self.current.y as f32) / (self.size as f32)
	}
	*/
}
