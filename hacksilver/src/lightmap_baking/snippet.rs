use super::internal::*;

// #[derive(Default)]
// pub struct Snippet1 {
// 	pub face: Face,
// 	pub img: BorderedImg,
// }
//
// impl Snippet1 {
// 	pub fn new(face: &Face, inner_size: uvec2) -> Self {
// 		Self {
// 			face: face.clone(),
// 			img: BorderedImg::new_with_inner_size(inner_size),
// 		}
// 	}
// }

/// Image with border
#[derive(Default)]
pub struct BorderedImg {
	img: Img<Color>,
}

impl BorderedImg {
	pub const SNIPPET_MARGIN: u32 = 2;

	pub fn new_with_inner_size(inner_size: uvec2) -> Self {
		let outer_size = inner_size + uvec2(2 * Self::SNIPPET_MARGIN, 2 * Self::SNIPPET_MARGIN);
		Self::new_with_outer_size(outer_size)
	}

	fn new_with_outer_size(outer: uvec2) -> Self {
		// spot-check check for accidentally passing inner size:
		// outer size has 2pix margin at each side.
		assert!(outer.iter().all(|v| v > 4));

		Self { img: Img::new(outer) }
	}

	pub fn from_img_with_margin(img: Img<Color>) -> Self {
		Self { img }
	}

	pub fn from_img_without_margin(img: Img<Color>) -> Self {
		let mut new = Self::new_with_inner_size(img.size());
		let off = uvec2(Self::SNIPPET_MARGIN, Self::SNIPPET_MARGIN);
		new.img.draw(off, &img);
		new
	}

	// refactor scaffolding. TODO: remove
	pub fn img_with_margin(&self) -> &Img<Color> {
		&self.img
	}

	pub fn outer_size(&self) -> uvec2 {
		self.img.size()
	}

	pub fn inner_size(&self) -> uvec2 {
		self.outer_size() - 2 * uvec2(Self::SNIPPET_MARGIN, Self::SNIPPET_MARGIN)
	}

	pub fn at_inner_idx(&self, p: uvec2) -> Color {
		self.img.at(p + uvec2(Self::SNIPPET_MARGIN, Self::SNIPPET_MARGIN))
	}

	pub fn set_inner_idx(&mut self, p: uvec2, c: Color) {
		self.img.set(p + uvec2(Self::SNIPPET_MARGIN, Self::SNIPPET_MARGIN), c)
	}

	pub fn at_uv_no_margin(&self, uv: vec2) -> Color {
		Self::at_uv_with_margin(self, uv, Self::SNIPPET_MARGIN)
	}

	// Index an image at UV coordinates.
	// refactor scaffolding. TODO: remove
	fn at_uv_with_margin(img: &BorderedImg, uv: vec2, margin: u32) -> Color {
		let inner_size = img.outer_size() - 2 * uvec2(margin, margin);
		let pix = linterp(vec2(0.0, 0.0), vec2(0.5, 0.5), vec2(1.0, 1.0), inner_size.to_f32() - vec2(0.5, 0.5), uv).map(|v| v.floor() as u32);
		img.img_with_margin().at(pix + uvec2(margin, margin))
	}

	// This way, we can clearly see if we accidentally index into the margin.
	pub fn draw_margin(&mut self) {
		let img = &mut self.img;
		const GREEN: Color = vec3(0.0, 1.0, 0.0);
		const YELLOW: Color = vec3(1.0, 1.0, 0.0);
		const BLUE: Color = vec3(0.0, 0.0, 1.0);
		const RED: Color = vec3(1.0, 0.0, 0.0);
		let (w, h) = img.size().into();
		img.pixels_mut().iter_mut().for_each(|p| *p = GREEN); // inner pixels, visible
		draw_rect(img, (2, w - 3), (2, h - 3), YELLOW); // face edge, half visible
		draw_rect(img, (1, w - 2), (1, h - 2), BLUE); // margin, 1st pixel, outside of face but bleeds through via linear interpolation
		draw_rect(img, (0, w - 1), (0, h - 1), RED); // margin, 2nd pixel, fully invisible bit might bleed due to round-off
	}

	pub fn sum_2<'a, I1, I2>(a: I1, b: I2) -> impl Iterator<Item = Self> + 'a
	where
		I1: IntoIterator<Item = &'a Self> + 'a,
		I2: IntoIterator<Item = &'a Self> + 'a,
	{
		a.into_iter().zip(b.into_iter()).map(|(a, b)| Self {
			img: Img::from_fn(a.outer_size(), |pix| a.img.at(pix) + b.img.at(pix)),
		})
	}

	pub fn sum_3<'a, I1, I2, I3>(a: I1, b: I2, c: I3) -> impl Iterator<Item = Self> + 'a
	where
		I1: IntoIterator<Item = &'a Self> + 'a,
		I2: IntoIterator<Item = &'a Self> + 'a,
		I3: IntoIterator<Item = &'a Self> + 'a,
	{
		a.into_iter().zip(b.into_iter()).zip(c.into_iter()).map(|((a, b), c)| Self {
			img: Img::from_fn(a.outer_size(), |pix| a.img.at(pix) + b.img.at(pix) + c.img.at(pix)),
		})
	}
}

// How big should the lightmap image of a Face be?
// Baking resolution * face physical size (orientation-independent).
//fn lightmap_size_with_margin(cfg: &BakeOpts, face: &Face) -> uvec2 {
//	lightmap_size_no_margin(cfg, face) + (2 * SNIPPET_MARGIN) * uvec2::ONES
//}

// Color an image's outer pixels, which are supposed to be
fn draw_rect<C>(img: &mut Img<C>, (x1, x2): (u32, u32), (y1, y2): (u32, u32), color: C)
where
	C: Copy + Default,
{
	for x in x1..=x2 {
		img.set((x, y1), color);
		img.set((x, y2), color);
	}
	for y in y1..=y2 {
		img.set((x1, y), color);
		img.set((x2, y), color);
	}
}

pub struct TmpSnip {}

/// A Face + baked lightmap image.
/// Baked snippets are to be
///  1) positioned on a lightmap atlas
///  2) assembled into a Vertex Array
/// by zonegraph.rs.
pub struct Snippet2 {
	pub face: Face,
	pub direct_visibility: RgbImage,
	pub ambient: RgbImage,
}

impl Snippet2 {
	pub fn dimensions(&self) -> uvec2 {
		debug_assert!(self.direct_visibility.dimensions() == self.ambient.dimensions());
		self.direct_visibility.dimensions().into()
	}
}

// types to increase readability
type Color = vec3;
