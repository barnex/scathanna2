use super::internal::*;

pub trait App: Send + 'static {
	/// Called periodically by the `Shell`.
	///
	/// The App must advance its state forward in time,
	/// given the inputs since the last tick.
	///
	/// The App may request an Shell state change (e.g. exit).
	fn handle_tick(&mut self, inputs: &Inputs) -> StateChange;

	/// Called by the `Shell` when a redraw is needed.
	fn handle_draw(&self, viewport_size: uvec2) -> SceneGraph;

	/// The `Shell` has a text console where commands can be typed.
	/// Commands are passed here.
	fn handle_command(&mut self, _cmd: &str) -> Result<()> {
		Err(anyhow!("commands not supported"))
	}
}
