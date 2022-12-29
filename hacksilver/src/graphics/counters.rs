use std::{sync::atomic::AtomicU64, time::SystemTime};

use super::internal::*;

#[derive(Default)]
pub struct Counters {
	pub unix_micros: AtomicU64,
	pub draw_calls: Counter,
	pub draw_instances: Counter,
	pub buffer_creates: Counter,
	pub buffer_uploads: Counter,
	pub bytes_uploaded: Counter,
	pub texture_uploads: Counter,
	pub vertices: Counter,
}

impl Counters {
	/// Return a string displaying per-frame statistics
	/// (frames per second, number of texture uploads, ...)
	/// since the previous call to `format_and_reset`.
	///
	/// I.e., this method needs to be called exactly once per rendered frame.
	pub fn format_and_reset(&self) -> String {
		let new_micros = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("system time").as_micros() as u64;
		let prev_micros = self.unix_micros.swap(new_micros, std::sync::atomic::Ordering::SeqCst);
		let delta_micros = new_micros - prev_micros;
		let delta_secs = (delta_micros as f32) / 1e6;

		let fps = 1.0 / delta_secs;
		let draw = self.draw_calls.take();
		let instances = self.draw_instances.take();
		let vertices = self.vertices.take();
		let buf_new = self.buffer_creates.take();
		let buf_tx = self.buffer_uploads.take();
		let bytes_tx = self.bytes_uploaded.take();
		let tex_tx = self.texture_uploads.take();

		format!(
			r"      FPS: {fps:.1}
     draw: {draw}
instances: {instances}
 vertices: {vertices}
  buf new: {buf_new}
   buf tx: {buf_tx}
 bytes tx: {bytes_tx}
   tex tx: {tex_tx}"
		)
	}
}
