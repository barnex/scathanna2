use super::internal::*;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug, Hash)]
#[repr(u8)]
pub enum Team {
	Red = 0,
	Blue = 1,
	Green = 2,
}

pub const NUM_TEAMS: usize = 3;

//use Team::*;
impl Team {
	/// To be multiplied by colors to make them team-color like.
	pub fn color_filter(self) -> vec3 {
		match self {
			Team::Red => vec3(1.0, 0.5, 0.5),
			Team::Blue => vec3(0.5, 0.5, 1.0),
			Team::Green => vec3(0.5, 1.0, 0.3),
		}
	}
}

impl FromStr for Team {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self> {
		use Team::*;
		match s {
			"red" => Ok(Red),
			"blu" | "blue" => Ok(Blue),
			"green" => Ok(Green),
			bad => Err(anyhow!("unknown team `{}`, options: `red`, `blue`, `green`", bad)),
		}
	}
}

impl fmt::Display for Team {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Team::Red => f.write_str("Red"),
			Team::Blue => f.write_str("Blue"),
			Team::Green => f.write_str("Green"),
		}
	}
}
