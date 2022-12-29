use super::internal::*;

pub struct TextConsole<A: App> {
	ctx: Arc<GraphicsCtx>,

	typing: Option<String>,
	error: Option<String>,
	child: A,
}

impl<A: App> TextConsole<A> {
	pub fn new(ctx: &Arc<GraphicsCtx>, child: A) -> Self {
		Self {
			ctx: ctx.clone(),
			child,
			typing: None,
			error: None,
		}
	}

	fn draw(&self, viewport_size: uvec2) -> SceneGraph {
		let mut scenegraph = self.child.handle_draw(viewport_size);

		if let Some(cmd) = &self.typing {
			// show the CLI
			let cli_text = Object::new(
				&Arc::new(self.ctx.upload_meshbuffer(&layout_text_bottom(viewport_size, &format!(">{cmd}")))),
				self.ctx.shader_pack.text(),
			);
			scenegraph.objects.push(cli_text);

			// show log when typing in the CLI
			let log_text = LOG.to_string();
			let pos = uvec2(0, viewport_size.y() - EMBEDDED_CHAR_SIZE.y() - text_size_pix(&log_text).y());
			let log_text = Object::new(&Arc::new(self.ctx.upload_meshbuffer(&layout_text(viewport_size, pos, &log_text))), self.ctx.shader_pack.text());
			scenegraph.objects.push(log_text);
		} else if let Some(err) = &self.error {
			// show the error returned by the last command
			//let text = Object::new(&Arc::new(self.ctx.upload_meshbuffer(&layout_text_bottom(viewport_size, err))), self.ctx.shader_pack.text());
			//scenegraph.objects.push(text);
		}

		scenegraph
	}

	fn tick(&mut self, inputs: &Inputs) -> StateChange {
		match self.typing {
			None => self.tick_normal(inputs),
			Some(_) => {
				self.tick_typing(inputs);
				StateChange::None
			}
		}
	}

	fn tick_normal(&mut self, inputs: &Inputs) -> StateChange {
		if inputs.is_pressed(Button::CONSOLE) {
			self.typing = Some(String::new());
			StateChange::None
		} else {
			self.child.handle_tick(inputs)
		}
	}

	fn tick_typing(&mut self, inputs: &Inputs) {
		// exit typing mode on ESC
		if inputs.is_pressed(Button::ESC) || inputs.is_pressed(Button::CONSOLE) {
			self.typing = None;
			self.error = None;
			return;
		}

		//let cmd = self.typing.as_mut().expect("in typing mode");
		// append typed characters
		for chr in inputs.received_characters().chars() {
			match chr {
				'\x08' => drop(self.cmd_mut().pop()), // backspace
				'\r' => {
					let cmd = self.typing.take().unwrap();
					let cmd = cmd.trim();
					if !cmd.is_empty() {
						let result = self.child.handle_command(&cmd);
						LOG.write(format!(">{}: {:?}", cmd, result));
						if let Err(e) = result {
							self.error = Some(format!("Error: {}", e));
						} else {
							self.error = None
						}
					} else {
						// empty command: print newline
						LOG.write("");
					}
				}
				chr => {
					if !chr.is_ascii_control() {
						self.cmd_mut().push(chr)
					}
				}
			}
		}
	}

	fn cmd_mut(&mut self) -> &mut String {
		self.typing.as_mut().expect("must be in typing mode")
	}
}

impl<A: App> App for TextConsole<A> {
	fn handle_draw(&self, viewport_size: uvec2) -> SceneGraph {
		self.draw(viewport_size)
	}

	fn handle_tick(&mut self, inputs: &Inputs) -> StateChange {
		self.tick(inputs)
	}
}
