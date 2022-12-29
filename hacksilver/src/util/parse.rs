use super::internal::*;

// Loose bool parsing (on/off/true/false).
pub fn parse_bool(v: &str) -> Result<bool> {
	match v {
		"on" => Ok(true),
		"off" => Ok(false),
		v => Ok(v.parse()?),
	}
}
