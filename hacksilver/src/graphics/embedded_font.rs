use super::internal::*;

/// Size (in pixels) of a single character in the embedded font map.
pub const EMBEDDED_CHAR_SIZE: uvec2 = uvec2(8, 16);

/// Overall size (in pixels) of the embedded font map.
pub const EMBEDDED_FONTMAP_SIZE: uvec2 = uvec2(128, 128);

/// Rendering this "string" gives a crosshair.
pub const EMBEDDED_CROSSHAIR: &str = "\x06\x07";
