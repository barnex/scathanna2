use super::internal::*;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default)]
pub struct SpawnPoint {
	pub pos: ivec3,
	pub yaw: f32,
}

impl SpawnPoint {
	pub fn position(&self) -> vec3 {
		self.pos.to_f32()
	}

	pub fn orientation(&self) -> Orientation {
		Orientation { yaw: self.yaw, pitch: 0.0 }
	}
}
