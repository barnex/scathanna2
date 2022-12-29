use super::internal::*;
use winit::{
	dpi::PhysicalPosition,
	event::{MouseButton, MouseScrollDelta},
};

/// Accumulates input events since the last tick,
/// allowing for queries like "is this key currently held down?".
///
/// Also de-bounces events faster than a tick,
/// and removes OS key repeats.
#[derive(Default, Debug)]
pub struct Inputs {
	pub buttons_down: Set<Button>,
	pub buttons_pressed: Set<Button>,
	pub buttons_released: Set<Button>,
	pub received_characters: String,
	pub mouse_delta: ivec2,
	pub tick_time: Duration,
}

/// A keystroke or mouse click or scroll action
/// are all uniformly treated as "button" pushes.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Button {
	Key(VirtualKeyCode),
	Mouse(MouseButton),
	MouseWheelUp_, // TODO: replace entirely by scroll delta
	MouseWheelDown_,
}

impl Button {
	pub const MOUSE1: Self = Self::Mouse(winit::event::MouseButton::Left);
	pub const MOUSE2: Self = Self::Mouse(winit::event::MouseButton::Right);
	pub const PAINT: Self = Self::Mouse(winit::event::MouseButton::Other(9));
	pub const ESC: Self = Self::Key(VirtualKeyCode::Escape);
	pub const CONSOLE: Self = Self::Key(VirtualKeyCode::Tab);
	pub const GRAB: Self = Self::Key(VirtualKeyCode::G);
	pub const ROTATE: Self = Self::Key(VirtualKeyCode::R);
	pub const ROTATE2: Self = Self::Key(VirtualKeyCode::T);
	pub const BLOCKTYP: Self = Self::Key(VirtualKeyCode::B);
	pub const TEXTURE: Self = Self::Key(VirtualKeyCode::Q);
	pub const ADD: Self = Self::Key(VirtualKeyCode::W);
	pub const COPY: Self = Self::Key(VirtualKeyCode::C);
	pub const PASTE: Self = Self::Key(VirtualKeyCode::V);
	pub const DELETE: Self = Self::Key(VirtualKeyCode::X);
	pub const FORWARD: Self = Self::Key(VirtualKeyCode::E);
	pub const LEFT: Self = Self::Key(VirtualKeyCode::S);
	pub const BACKWARD: Self = Self::Key(VirtualKeyCode::D);
	pub const RIGHT: Self = Self::Key(VirtualKeyCode::F);
	pub const JUMP: Self = Self::Key(VirtualKeyCode::Space);
	pub const CROUCH: Self = Self::Key(VirtualKeyCode::Z);
	pub const SHIFT: Self = Self::Key(VirtualKeyCode::LShift);
	pub const CONTROL: Self = Self::Key(VirtualKeyCode::LControl);
	pub const ALT: Self = Self::Key(VirtualKeyCode::LAlt);
}

impl Inputs {
	/// Time since the last tick, in seconds.
	/// This tells us how long buttons were pressed.
	pub fn dt(&self) -> f32 {
		self.tick_time.as_secs_f32()
	}

	/// Forget all pending inputs.
	/// (E.g. needed after focus loss on Wayland:
	/// ESC DOWN gets recorded but ESC UP not (X11 sends both))
	pub fn clear(&mut self) {
		*self = default()
	}

	/// Is a button currently held down?
	/// (This repeats on every tick for as long as the button is held)
	pub fn is_down(&self, but: Button) -> bool {
		self.buttons_down.contains(&but)
	}

	/// Was a button pressed right before the current tick?
	/// This triggers only once per physical keypress.
	/// OS keyboard repeats are ignored.
	pub fn is_pressed(&self, but: Button) -> bool {
		self.buttons_pressed.contains(&but)
	}

	/// Was a button released right before the current tick?
	pub fn is_released(&self, but: Button) -> bool {
		self.buttons_released.contains(&but)
	}

	/// Iterate over all keys currently held down.
	pub fn buttons_down(&self) -> impl Iterator<Item = Button> + '_ {
		self.buttons_down.iter().copied()
	}

	/// Iterate over all keys pressed down right before this tick.
	pub fn buttons_pressed(&self) -> impl Iterator<Item = Button> + '_ {
		self.buttons_pressed.iter().copied()
	}

	/// Iterate over all keys released right before this tick.
	pub fn buttons_released(&self) -> impl Iterator<Item = Button> + '_ {
		self.buttons_released.iter().copied()
	}

	/// The button that was pressed during the last tick, assuming there was only one.
	/// (More than one pressed causes the superfluous ones to be dropped arbitrarily).
	/// Used for the editor where pressing two buttons at the same time is rare and useless.
	pub fn pressed_button(&self) -> Option<Button> {
		self.buttons_pressed.iter().next().copied()
	}

	/// The relative mouse movement since the last tick.
	pub fn mouse_delta(&self) -> vec2 {
		self.mouse_delta.convert()
	}

	/// The relative mouse wheel movement since last tick.
	pub fn mouse_wheel_delta(&self) -> i32 {
		let mut delta = 0;
		if self.is_pressed(Button::MouseWheelDown_) {
			delta -= 1;
		}
		if self.is_pressed(Button::MouseWheelUp_) {
			delta += 1;
		}
		delta
	}

	/// The unicode characters typed since the last tick.
	pub fn received_characters(&self) -> &str {
		&self.received_characters
	}

	/// Forget all changes since previous `forget` call.
	/// Called by the event loop after the inputs have been used. I.e. after each tick.
	pub fn forget(&mut self) {
		self.buttons_pressed.clear();
		self.buttons_released.clear();
		self.mouse_delta = ivec2(0, 0);
		self.received_characters.clear();
		self.tick_time = Duration::ZERO;
	}

	/// Record that an event happened since the last `forget` call
	/// (i.e. since the last tick).
	/// (Called by the event loop, not to be called by the event consumer (i.e. App)).
	pub fn record_window_event(&mut self, event: &WindowEvent) {
		use WindowEvent::*;
		match event {
			CloseRequested => (),
			ReceivedCharacter(chr) => self.received_characters.push(*chr),
			KeyboardInput {
				input: winit::event::KeyboardInput {
					state,
					virtual_keycode: Some(virtual_keycode),
					..
				},
				..
			} => self.record_button(Button::Key(*virtual_keycode), *state),
			ModifiersChanged(_) => (),
			MouseInput { state, button, .. } => self.record_button(Button::Mouse(*button), *state),
			WindowEvent::MouseWheel { delta, .. } => self.record_mouse_wheel(delta),
			_ => (),
		}
	}

	/// Record mouse motion.
	/// All mouse motion between ticks is added up,
	/// and presented as a single motion.
	/// (Called by the event loop, not to be called by the event consumer (i.e. App)).
	pub fn record_mouse_motion(&mut self, delta: dvec2) {
		let delta = delta.convert::<i32>();
		self.mouse_delta += delta;
	}

	/// Record a key or mouse button event (handled uniformly).
	pub fn record_button(&mut self, but: Button, state: ElementState) {
		use ElementState::*;
		match state {
			Pressed => {
				if !self.buttons_down.contains(&but) {
					self.buttons_down.insert(but);
					// only record as pressed if not down yet
					// to remove key repeats.
					self.buttons_pressed.insert(but);
				}
			}
			Released => {
				self.buttons_down.remove(&but);
				// do not removed from pressed (yet)
				// a button can be pressed and released within the same tick.
				self.buttons_released.insert(but);
			}
		}
	}

	fn record_mouse_wheel(&mut self, delta: &MouseScrollDelta) {
		/*
			Mouse wheel delta's can vary wildly,
			reduce them just a single Up / Down event
			discarding the scroll amount.
		*/
		let dy = match delta {
			MouseScrollDelta::LineDelta(_, y) => *y,
			MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => *y as f32,
		};
		let button = match dy {
			_ if dy > 0.0 => Some(Button::MouseWheelUp_),
			_ if dy < 0.0 => Some(Button::MouseWheelDown_),
			_ => None,
		};
		/*
			Record both a press and release
			to make the scroll event appear as a button press
			(the scroll wheel cannot be "held down" continuously like a mouse button).
		*/
		if let Some(button) = button {
			self.buttons_pressed.insert(button);
			self.buttons_released.insert(button);
		}
	}
}
