use super::internal::*;

// Like Object, but on the host.
#[derive(Serialize, Deserialize)]
pub struct HObject {
	pub meshbuf: MeshBuffer,
	pub mat_id: MatID,
}
