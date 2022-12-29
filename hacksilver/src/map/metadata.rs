use super::internal::*;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Metadata {
	#[serde(default)]
	pub spawn_points: Vec<SpawnPoint>,

	#[serde(default)]
	pub pickup_points: Vec<PickupPoint>,

	#[serde(default = "default_sun_dir")]
	pub sun_dir: vec3,

	#[serde(default)]
	pub sun_color: vec3,

	#[serde(default)]
	pub sky_color: vec3,
}

fn default_sun_dir() -> vec3 {
	vec3(0.304855380424846, 0.609710760849692, 0.731652913019631).normalized()
}

impl Metadata {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn save(&self, map_dir: &MapDir) -> Result<()> {
		let file = &map_dir.metadata_file();
		Ok(serde_json::to_writer_pretty(create(file)?, self)?)
	}

	pub fn load(map_dir: &MapDir) -> Result<Self> {
		let file = &map_dir.metadata_file();
		Ok(serde_json::from_reader::<_, Metadata>(open(file)?)? //
			.with(|md| md.sun_dir.normalize()))
	}
}
