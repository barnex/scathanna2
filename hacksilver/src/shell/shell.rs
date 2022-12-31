//#[allow(deprecated)]

use std::time::Instant;

use super::internal::*;
use winit::dpi::LogicalSize;
use winit::event::*;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::CursorGrabMode;
use winit::window::WindowBuilder;

/// The Shell is a "Graphics Terminal".
///
/// Like a Unix terminal runs a program,
/// (sending it text input and rendering the text output),
/// a graphics `Shell` runs an `App` (e.g. GameState, Editor),
/// sends it user input, and renders the 3D output.
///
/// +---------------------+
/// |    graphics shell   |
/// +---------------------+
/// |                     |    user inputs    App{
/// |        ___          |   ------------->   handle_tick()
/// |      /___/|         |
/// |      |___|/         |    scenegraph
/// |                     |   <-------------   handle_draw()
/// +---------------------+                  }
///
///
/// In addition, the `Shell` provides some utilities like
/// logging, error reporting, text input, and an FPS counter.
pub struct Shell {
	window: Window,
	canvas: Canvas,

	cursor_grabbed: bool,
	previous_tick: Instant,
	input_state: Inputs,
	app: Box<dyn App + Send>,
}

impl Shell {
	/// Open a Shell window that will construct and run an `App`. E.g.:
	///
	///   Shell::main_loop(Game::new)
	///
	/// Where `Game::new` constructs a new game state.
	///
	/// It is fine for your constructor function to take a lot of time
	/// (e.g. loading assets etc). Until it returns, the Shell will
	/// display "Loading..." and remain responsive.
	pub fn main_loop<F, A>(opts: GraphicsOpts, new_app: F) -> Result<()>
	where
		A: App,
		F: FnOnce(&Arc<GraphicsCtx>) -> Result<A> + Send + 'static,
	{
		let event_loop = EventLoop::new();
		let window = WindowBuilder::new() //
			.with_inner_size(LogicalSize::<u32> {
				width: opts.width,
				height: opts.height,
			})
			.with_fullscreen(match opts.fullscreen {
				true => Some(winit::window::Fullscreen::Borderless(None)),
				false => None,
			})
			.with_title("hacksilver engine")
			.build(&event_loop)?;

		let canvas = Canvas::new(opts, &window)?;
		let ctx = canvas.graphics_context();

		let loading_screen = LoadingScreen::new(ctx, |ctx| Ok(TextConsole::new(ctx, new_app(ctx)?)));
		let app = Box::new(loading_screen);

		let shell = Self {
			cursor_grabbed: false,
			app,
			previous_tick: Instant::now(),
			input_state: default(),
			canvas,
			window,
		};

		write_splash_screen();
		Ok(shell.event_loop(event_loop))
	}

	fn event_loop(mut self, event_loop: EventLoop<()>) {
		let my_window_id = self.window.id();
		event_loop.run(move |event, _, control_flow| {
			match event {
				Event::WindowEvent { ref event, window_id } if window_id == my_window_id => {
					//println!("WindowEvent: {:?}", &event);
					if self.cursor_grabbed {
						self.handle_window_event(event);
					}
					match event {
						WindowEvent::MouseInput { .. } => self.grab_cursor(),
						WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
						WindowEvent::Resized(physical_size) => {
							self.handle_resize(*physical_size);
						}
						WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
							self.handle_resize(**new_inner_size);
						}
						WindowEvent::Focused(false) => self.release_cursor(),
						_ => {}
					};
				}
				Event::DeviceEvent { event, .. } => {
					//println!("DeviceEvent: {:?}", &event);
					match event {
						DeviceEvent::MouseMotion { delta } => {
							if self.cursor_grabbed {
								self.handle_mouse_motion(delta.into())
							}
						}
						// Always handle ESC regardless of focus, so we don't steal the cursor.
						DeviceEvent::Key(
							KeyboardInput {
								virtual_keycode: Some(VirtualKeyCode::Escape),
								state,
								..
							},
							..,
						) => self.input_state.record_button(Button::Key(VirtualKeyCode::Escape), state),
						_ => (),
					}
				}
				Event::RedrawRequested(window_id) if window_id == my_window_id => {
					// Note: without testing for ControlFlow::Exit,
					// closing the window *sometimes* hangs in what appears to be
					// a race condition (not *data* race) between exit and an pending redraw.
					if *control_flow != ControlFlow::Exit {
						self.handle_request_redraw();
					}
				}
				Event::MainEventsCleared => {
					if *control_flow != ControlFlow::Exit {
						self.window.request_redraw(); // Continuously draw
					}
				}
				_ => {}
			}
		});
	}
}

impl Shell {
	pub fn viewport_size(&self) -> uvec2 {
		self.canvas.viewport_size()
	}

	// Attempt to grab the mouse cursor if not yet grabbed.
	fn grab_cursor(&mut self) {
		if !self.cursor_grabbed {
			self.window.set_cursor_visible(false);
			// MacOSX hack
			let _ = self.window.set_cursor_grab(CursorGrabMode::Locked);
			match self.window.set_cursor_grab(CursorGrabMode::Confined) {
				Ok(()) => {
					println!("Mouse cursor grabbed. Press ESC to release.");
					self.cursor_grabbed = true;
				}
				Err(e) => {
					log::error!("grab cursor: {}", e);
				}
			}
		}
	}

	// Release the mouse cursor if grabbed.
	fn release_cursor(&mut self) {
		if self.cursor_grabbed {
			self.window.set_cursor_visible(true);
			match self.window.set_cursor_grab(CursorGrabMode::None) {
				Ok(()) => (),
				Err(e) => log::error!("release cursor: {}", e),
			}
		}
		self.cursor_grabbed = false;
		// Needed after focus loss on Wayland:
		// ESC DOWN gets recorded but ESC UP not (X11 sends both).
		self.input_state.clear();
	}

	fn handle_request_redraw(&mut self) {
		self.tick();
		self.redraw();
	}

	fn tick(&mut self) {
		self.update_dt();
		let app = &mut self.app;
		let state_change = app.handle_tick(&self.input_state);
		self.input_state.forget();

		use StateChange::*;
		match state_change {
			None => (),
			ReleaseCursor => self.release_cursor(),
		}
	}

	/// Update the current time step, in preparation of a new `tick` call.
	fn update_dt(&mut self) {
		const MIN_DT: Duration = Duration::from_millis(1);
		const MAX_DT: Duration = Duration::from_millis(100);
		let now = Instant::now();
		self.input_state.tick_time = (now - self.previous_tick).clamp(MIN_DT, MAX_DT);
		self.previous_tick = now;
	}

	fn redraw(&mut self) {
		let scene = self.app.handle_draw(self.canvas.viewport_size());
		self.canvas.render(scene);
	}

	fn handle_resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
		if size.width > 0 && size.height > 0 {
			self.canvas.resize(uvec2(size.width, size.height));
		}
	}

	fn handle_window_event(&mut self, event: &WindowEvent) {
		self.input_state.record_window_event(event);
	}

	fn handle_mouse_motion(&mut self, delta: dvec2) {
		self.input_state.record_mouse_motion(delta);
	}
}

fn write_splash_screen() {
	#[cfg(debug_assertions)]
	LOG.write(
		"*******
WARNING: running in debug mode.
Be sure to compile with '--release' for better performance.
*******",
	);
}
