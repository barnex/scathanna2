//use super::internal::*;

/// Returned by an `App` to request a change in the `Shells`'s state machine.
/// TODO: remove, give direct control to shell (grab/release, close, fullscreen, ...)
pub enum StateChange {
	/// Keep the current App
	None,

	/// Release the mouse cursor
	ReleaseCursor,
}
