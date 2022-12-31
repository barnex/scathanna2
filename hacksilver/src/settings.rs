use super::internal::*;

/// All user-controlled settings, read from "settings.toml".
#[derive(Deserialize)]
pub struct Settings {
	pub graphics: GraphicsOpts,
	pub controls: Controls,
	pub player: PlayerOpts,
	pub sound: SoundOpts,
	pub network: NetworkOpts,
}

#[derive(Deserialize, Clone)]
pub struct Controls {
	pub forward: char,
	pub left: char,
	pub backward: char,
	pub right: char,
	pub jump: char,
	pub mouse_sensitivity: f32,
}

impl Default for Controls {
	fn default() -> Self {
		Self {
			forward: 'w',
			left: 'a',
			backward: 's',
			right: 'd',
			jump: ' ',
			mouse_sensitivity: 100.0,
		}
	}
}

#[derive(Deserialize)]
pub struct PlayerOpts {
	pub name: String,
	pub avatar: u8,
	pub team: String,
}

#[derive(Deserialize)]
pub struct SoundOpts {
	pub enabled: bool,
}

#[derive(Deserialize)]
pub struct NetworkOpts {
	pub servers: Vec<String>,
}

pub fn load_toml<T: DeserializeOwned>(path: &Path) -> Result<T> {
	LOG.write(format!("load settings: {path:?}"));
	let mut f = open(path)?;
	let mut buf = String::new();
	f.read_to_string(&mut buf)?;
	toml::from_str(&buf).map_err(|e| anyhow!("load settings: {e:#}"))
}
