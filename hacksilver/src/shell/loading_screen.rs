use super::internal::*;

/// An App that renders a loading message while loading another App in the background.
pub struct LoadingScreen<A: App> {
	ctx: Arc<GraphicsCtx>,
	state: State<A>,
}

enum State<A: App> {
	Loading(Receiver<Result<A>>),
	Ready(A),
	Errored,
}

impl<A: App> LoadingScreen<A> {
	pub fn new<F>(ctx: &Arc<GraphicsCtx>, work: F) -> Self
	where
		F: FnOnce(&Arc<GraphicsCtx>) -> Result<A> + Send + 'static,
	{
		let (send, recv) = mpsc::channel();

		{
			let ctx = ctx.clone();
			spawn(move || {
				let result = work(&ctx);
				log::info!("loading done");
				send.send(result).unwrap_or_else(|err| log::error!("{}", err));
				// Note: send error only occurs if receiver dropped, so main program must already be terminating.
			});
		}

		Self {
			ctx: ctx.clone(),
			state: State::Loading(recv),
		}
	}

	fn draw_loading(&self, viewport_size: uvec2) -> SceneGraph {
		let text = Object::new(
			&Arc::new(self.ctx.upload_meshbuffer(&layout_text(viewport_size, uvec2(0, 0), &LOG.to_string()))),
			self.ctx.shader_pack.text(),
		);
		SceneGraph::new(viewport_size).with(|sg| {
			sg.bg_color = vec3(0.02, 0.04, 0.08);
			sg.push(text);
		})
	}
}

impl<A: App> App for LoadingScreen<A> {
	fn handle_draw(&self, viewport_size: uvec2) -> SceneGraph {
		match &self.state {
			State::Loading(_) | State::Errored => self.draw_loading(viewport_size),
			State::Ready(a) => a.handle_draw(viewport_size),
		}
	}

	// on every tick, check if the App under construction has finished loading.
	fn handle_tick(&mut self, inputs: &Inputs) -> StateChange {
		if inputs.is_pressed(Button::Key(VirtualKeyCode::Escape)) {
			return StateChange::ReleaseCursor;
		}

		match &mut self.state {
			State::Errored => StateChange::None,
			State::Loading(recv) => match recv.try_recv() {
				Ok(Ok(new_app)) => {
					self.state = State::Ready(new_app);
					StateChange::None
				}
				Ok(Err(app_err)) => {
					self.state = State::Errored;
					LOG.write(format!("ERROR: {}", app_err));
					StateChange::None
				}
				Err(mpsc::TryRecvError::Empty) => StateChange::None,
				Err(mpsc::TryRecvError::Disconnected) => panic!("BUG"),
			},
			State::Ready(app) => app.handle_tick(inputs),
		}
	}
}
