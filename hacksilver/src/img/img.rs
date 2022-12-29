use super::internal::*;
use std::ops::{Index, IndexMut};

/// An Image is a rectangular 2D array of color values
/// (RGB, grayscale, ...)
#[derive(Debug, PartialEq, Clone)]
pub struct Img<C> {
	size: uvec2,
	values: Vec<C>,
}

impl<'a, C> Img<C> {
	pub fn from_fn<F>(size: uvec2, mut f: F) -> Self
	where
		F: FnMut(uvec2) -> C,
	{
		let mut values = Vec::with_capacity((size.x() * size.y()) as usize);
		let (w, h) = size.into();
		for iy in 0..h {
			for ix in 0..w {
				values.push(f(uvec2(ix, iy)));
			}
		}
		Self { size, values }
	}
}

impl<'a, C> Img<C>
where
	C: Default,
{
	/// new constructs an image with given width and height and filled with the default value.
	pub fn new(size: uvec2) -> Img<C> {
		Self::from_fn(size, |_| C::default())
	}
}

impl<'a, C> Img<C> {
	#[inline]
	/// width of the image, in pixels
	pub fn width(&self) -> u32 {
		self.size.x()
	}

	#[inline]
	/// height of the image, in pixels
	pub fn height(&self) -> u32 {
		self.size.y()
	}

	#[inline]
	/// width and height of the image
	pub fn size(&self) -> uvec2 {
		self.size
	}

	/// pixels in row-major order, iterable.
	pub fn pixels(&self) -> &[C] {
		&self.values
	}

	/// pixels in row-major order, iterable.
	pub fn pixels_mut(&mut self) -> &mut [C] {
		&mut self.values
	}

	pub fn map<F, T>(&self, f: F) -> Img<T>
	where
		F: Fn(&C) -> T,
	{
		Img::<T> {
			size: self.size,
			values: self.values.iter().map(f).collect(),
		}
	}

	pub fn mut_at<P: Into<uvec2>>(&mut self, p: P) -> &mut C {
		let p: uvec2 = p.into();
		&mut self[p.y() as usize][p.x() as usize]
	}

	pub fn ref_at<P: Into<uvec2>>(&self, p: P) -> &C {
		let p: uvec2 = p.into();
		&self[p.y() as usize][p.x() as usize]
	}
}

impl<'a, C> Img<C>
where
	C: Default + Copy,
{
	/// Draw img over this image.
	pub fn draw(&mut self, pos: uvec2, img: &Img<C>) {
		for y in 0..img.height() {
			for x in 0..img.width() {
				let dst = (pos.x() + x, pos.y() + y);
				if dst.0 < self.width() && dst.1 < self.height() {
					self.set(dst, img.at((x, y)));
				}
			}
		}
	}

	pub fn map_values<F, T>(&self, f: F) -> Img<T>
	where
		T: Copy + Default,
		F: Fn(C) -> T,
	{
		Img::<T> {
			size: self.size,
			values: self.values.iter().copied().map(f).collect(),
		}
	}

	pub fn at<P: Into<uvec2>>(&self, p: P) -> C {
		let p: uvec2 = p.into();
		self[p.y() as usize][p.x() as usize]
	}

	/// TODO: separate texture filtering
	pub fn at_uv_nearest_clamp<UV: Into<vec2>>(&self, uv: UV) -> C {
		let uv = uv.into();
		let pix = linterp(
			//
			vec2(0.0, 0.0), //
			vec2(0.0, 0.0), //
			vec2(1.0, 1.0),
			self.size.to_f32(), //
			uv,                 //
		)
		.map(|v| v.floor() as u32);
		let x = pix.x().clamp(0, self.size().x() - 1);
		let y = pix.y().clamp(0, self.size().y() - 1);
		self.at((x, y))
	}

	pub fn at_uv_nearest_wrap<UV: Into<vec2>>(&self, uv: UV) -> C {
		let uv = uv.into();

		let wrap = |v| {
			let mut v = v % 1.0;
			while v < 0.0 {
				v += 1.0;
			}
			// TODO: redundant?
			while v >= 1.0 {
				v -= 1.0;
			}
			v
		};

		let uv = uv.map(wrap);

		let pix = linterp(
			//
			vec2(0.0, 0.0), //
			vec2(0.0, 0.0), //
			vec2(1.0, 1.0),
			self.size.to_f32(), //
			uv,                 //
		)
		.map(|v| v.floor() as u32);
		//let x = pix.x().clamp(0, self.size().x() - 1); // should be impossible
		//let y = pix.y().clamp(0, self.size().y() - 1);
		self.at(pix)
	}

	#[inline]
	pub fn at_mut<P: Into<uvec2>>(&mut self, p: P) -> &mut C {
		let p: uvec2 = p.into();
		&mut self[p.y() as usize][p.x() as usize]
	}

	#[inline]
	pub fn set(&mut self, p: impl Into<uvec2>, c: C) {
		let p: uvec2 = p.into();
		self[p.y() as usize][p.x() as usize] = c;
	}

	#[inline]
	pub fn try_set(&mut self, p: impl Into<uvec2>, c: C) {
		let p: uvec2 = p.into();
		if p.x() < self.width() && p.y() < self.height() {
			self[p.y() as usize][p.x() as usize] = c;
		}
	}
	pub fn fill(&mut self, c: C) {
		self.pixels_mut().iter_mut().for_each(|p| *p = c)
	}
}

impl<'a, C> Img<C> {
	#[inline]
	pub fn try_at_i32(&self, p: ivec2) -> Option<&C> {
		if self.contains(p) {
			Some(&self[p.y() as usize][p.x() as usize])
		} else {
			None
		}
	}

	pub fn contains(&self, p: impl Into<ivec2>) -> bool {
		let p = p.into();
		let (x, y) = p.into();
		let w = self.width() as i32;
		let h = self.height() as i32;
		x >= 0 && x < w && y >= 0 && y < h
	}
}

impl<C> Default for Img<C>
where
	C: Copy + Default,
{
	fn default() -> Self {
		Self {
			size: uvec2(0, 0),
			values: Vec::new(),
		}
	}
}

impl<C> Index<usize> for Img<C> {
	type Output = [C];

	fn index(&self, i: usize) -> &[C] {
		let l = i * self.width() as usize;
		let h = l + self.width() as usize;
		&self.values[l..h]
	}
}

impl<C> IndexMut<usize> for Img<C> {
	fn index_mut(&mut self, i: usize) -> &mut [C] {
		let l = i * self.width() as usize;
		let h = l + self.width() as usize;
		&mut self.values[l..h]
	}
}

impl Img<vec3> {
	pub fn to_srgb(&self) -> RgbImage {
		RgbImage::from_fn(self.width(), self.height(), |x, y| Rgb(self.at((x, y)).map(linear_to_srgb).into()))
	}
}

// impl Img<Color> {
// 	/// Convert the image to raw BGRA bytes.
// 	/// Used to create SDL textures (brilliance-ui).
// 	//pub fn raw_bgra(&self) -> Vec<u8> {
// 	//	let (w, h) = self.dimensions();
// 	//	let mut raw = Vec::with_capacity((w * h * 4) as usize);
// 	//	for iy in 0..h {
// 	//		for ix in 0..w {
// 	//			let c = self[iy as usize][ix as usize].bgra();
// 	//			raw.extend_from_slice(&c);
// 	//		}
// 	//	}
// 	//	raw
// 	//}
//
// 	/// Convert the image to raw RGBA bytes.
// 	/// Used to create SDL textures (brilliance-ui).
// 	pub fn raw_rgba(&self) -> Vec<[u8; 4]> {
// 		let (w, h) = self.dimensions();
// 		let mut raw = Vec::with_capacity((w * h) as usize);
// 		for iy in 0..h {
// 			for ix in 0..w {
// 				raw.push(self[iy as usize][ix as usize].rgba())
// 			}
// 		}
// 		raw
// 	}
//
// 	/// Convert the image to raw RGB bytes.
// 	/// Used to save as image.
// 	pub fn raw_rgb(&self) -> Vec<u8> {
// 		let (w, h) = self.dimensions();
// 		let mut raw = Vec::with_capacity((w * h * 3) as usize);
// 		for iy in 0..h {
// 			for ix in 0..w {
// 				let c = self[iy as usize][ix as usize].srgb();
// 				raw.extend_from_slice(&c);
// 			}
// 		}
// 		raw
// 	}
// }
