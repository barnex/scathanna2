use super::internal::*;

/// A mesh for rendering text at a given position on the screen (using the embedded bitmap font).
/// Wraps long lines as shown below:
///
///   viewport size
///  +----------------+
///  |  `pos`+        |
///  |        your tex|
///  |        t here  |
///  |                |
///  +----------------+
///
pub fn layout_text(viewport_size: uvec2, pos: uvec2, text: &str) -> MeshBuffer {
	let char_size = EMBEDDED_CHAR_SIZE;

	let mut buf = MeshBuffer::new();

	let mut char_pos = pos;
	for &byte in text.as_bytes() {
		// newline
		if byte == b'\n' {
			char_pos[0] = pos.x();
			char_pos[1] += char_size.y();
			continue;
		}

		// wrap long lines
		if char_pos.x() > viewport_size.x() - char_size.x() {
			char_pos[0] = pos.x();
			char_pos[1] += char_size.y();
		}

		buf.append(&blit_chr(viewport_size, char_pos, byte));

		char_pos[0] += char_size.x();
	}

	buf
}

/// Like `layout_text`, but puts the text at the bottom left of the screen.
///
///    viewport size
///  +----------------+
///  |                |
///  |                |
///  |your text       |
///  |here            |
///  +----------------+
///
pub fn layout_text_bottom(scrn_pixels: uvec2, text: &str) -> MeshBuffer {
	let y = scrn_pixels.y() - EMBEDDED_CHAR_SIZE.y() * text_height_chars(text);
	let x = 0;
	let pos = uvec2(x, y);
	layout_text(scrn_pixels, pos, text)
}

pub fn layout_text_right(scrn_pixels: uvec2, text: &str) -> MeshBuffer {
	let y = 0;
	let x = scrn_pixels.x() - EMBEDDED_CHAR_SIZE.x() * text_width_chars(text);
	let pos = uvec2(x, y);
	layout_text(scrn_pixels, pos, text)
}

pub fn text_height_chars(text: &str) -> u32 {
	text.lines().count() as u32
}

pub fn text_width_chars(text: &str) -> u32 {
	text.lines().map(str::len).max().unwrap_or(0) as u32
}

pub fn text_size_chars(text: &str) -> uvec2 {
	uvec2(text_width_chars(text), text_height_chars(text))
}

pub fn text_size_pix(text: &str) -> uvec2 {
	uvec2(text_width_chars(text), text_height_chars(text)).mul(EMBEDDED_CHAR_SIZE)
}

/// A mesh for copying a single character to the screen.
fn blit_chr(scrn_pixels: uvec2, scrn_pos: uvec2, char: u8) -> MeshBuffer {
	let tex_pixels = EMBEDDED_FONTMAP_SIZE;
	let sprite_pixels = EMBEDDED_CHAR_SIZE;
	let tex_pos = chr_tex_pos_16x8(char, sprite_pixels);

	blit(tex_pixels, tex_pos, sprite_pixels, scrn_pixels, scrn_pos)
}

/// Pixel position (top-left corner) of an ascii character in the embedded font map.
fn chr_tex_pos_16x8(char: u8, sprite_pixels: uvec2) -> uvec2 {
	let x = (char & 0xf) as u32;
	let y = (char >> 4) as u32;
	uvec2(x, y) * sprite_pixels
}
