use super::internal::*;

pub struct Client {
	//menu: Menu, // TODO
	conn: Conn,

	state: ClientState,
	zones: ZoneGraph, // where to put this ?????
	eng: GameCtx,
	drawcfg: DrawCfg,

	dbg_overlay: bool,
	fps_overlay: bool,
}

type Conn = NetPipe<ClientMsg, ServerMsg>;

impl Client {
	pub fn new(ctx: &Arc<GraphicsCtx>, settings: Settings) -> Result<Self> {
		let server = settings
			.network
			.servers
			.get(0)
			.ok_or_else(|| anyhow!("no servers specified (flag --server or settings.toml: network > servers)"))?;

		let join_req = JoinRequest {
			name: settings.player.name.clone(),
			avatar_id: settings.player.avatar,
			team: settings.player.team.parse()?,
		};

		let (conn, acc) = Self::connect(server, join_req)?;

		let eng = GameCtx::new(ctx, settings)?;
		let (state, zones) = Self::load_state(&eng, acc.map_switch, acc.player_id)?;

		Ok(Self {
			eng,
			conn,
			state,
			zones,
			drawcfg: DrawCfg::default(),
			dbg_overlay: false,
			fps_overlay: false,
		})
	}

	fn load_state(eng: &GameCtx, map_switch: MapSwitch, player_id: ID) -> Result<(ClientState, ZoneGraph)> {
		let map_name = &map_switch.map_name;
		let map = Map::load(&eng.assets, &map_name)?;
		// load zonegraph from disk, or bake if not found
		let map_dir = eng.assets.map_dir(map_name);
		let hzones = HZoneGraph::load(&map_dir).unwrap_or_else(|err| {
			LOG.write(format!("ERROR loading zonegraph: {err}, falling back to live bake"));
			let todo = BakeOpts::default(); // TODO: replace by Engine !!!
			HZoneGraph::bake(&todo, &eng.resources.materials, map.data(), Cancel::new())
		});
		let zones = ZoneGraph::upload(&eng.graphics, &eng.resources.materials, &map.data().palette, hzones);
		let world = World::new(map, map_switch.entities);
		let state = ClientState::new(&eng, player_id, world);
		Ok((state, zones))
	}

	fn connect(server: &str, join_req: JoinRequest) -> Result<(Conn, AcceptedMsg)> {
		LOG.write(format!("Connecting to {server}..."));
		let mut tcp_stream = TcpStream::connect(&server)?;
		LOG.write(format!("Connected. Joining..."));
		wireformat::serialize_into(&mut tcp_stream, &join_req)?;
		let accepted_msg: AcceptedMsg = wireformat::deserialize_from(&mut tcp_stream) //
			.map_err(|e| anyhow!("reading accept message: {e}"))?;
		let player_id = accepted_msg.player_id;
		LOG.write(format!("Accepted as player {player_id}"));
		let conn = NetPipe::new(tcp_stream);
		Ok((conn, accepted_msg))
	}

	//--------------------------------------------------------------------------------  tick

	fn tick(&mut self, inputs: &Inputs) -> StateChange {
		//if self.conn.is_ok() {
		self.tick_connected(inputs).expect("TODO: Client: handle tick error");
		//} else {
		//self.tick_disconnected(inputs)
		//}

		if inputs.is_down(Button::ESC) {
			StateChange::ReleaseCursor
		} else {
			StateChange::None
		}
	}

	fn tick_connected(&mut self, inputs: &Inputs) -> Result<()> {
		while let Some(msg) = self.conn.try_recv() {
			match msg? {
				ServerMsg::SwitchMap(map_switch) => {
					let (state, zones) = Self::load_state(&self.eng, map_switch, self.state.player_id())?;
					self.state = state;
					self.zones = zones;
				}
				msg => self.state.apply_server_msg(&self.eng, msg),
			}
		}

		let diffs = self.state.tick(&self.eng, inputs);
		for diff in diffs {
			self.conn.send(diff)?;
		}

		if self.dbg_overlay {
			self.state.hud.set_text(HUDPos::TopLeft, &self.fmt_dbg_overlay(), 1.0);
		}
		if self.fps_overlay {
			self.state.hud.set_text(HUDPos::TopRight, &self.eng.graphics.dev.counters.format_and_reset(), 1.0);
		}

		Ok(())
	}

	fn tick_disconnected(&mut self, inputs: &Inputs) {
		//self.menu.show_info(format!("disconnected: {:?}", self.conn.error()))
		// TODO: poll for reconnection here / drop to menu
	}

	//--------------------------------------------------------------------------------  draw

	fn draw(&self, viewport_size: uvec2) -> SceneGraph {
		let mut sg = self.drawcfg.draw_gamestate(&self.eng, &self.zones, viewport_size, &self.state);

		//if self.dbg_overlay {
		//	self.draw_dbg_overlay(&mut sg)
		//}

		//if self.fps_overlay {
		//	self.draw_fps_overlay(&mut sg)
		//}

		sg
	}

	// debug overlay
	// TODO: in client
	fn fmt_dbg_overlay(&self) -> String {
		let state = &self.state;
		let world = &state.world;
		let player = state.local_player();
		let spawned = player.spawned;
		let position = player.skeleton.position;
		let look_dir = player.skeleton.orientation.look_dir();
		let on_ground = player.skeleton.on_ground(&self.state.world);
		let effects = world.entities.effects.len();

		format!(
			r#"
spawned: {spawned}
position: {position}
look_dir: {look_dir}
on_ground: {on_ground}
others: {}
effects: {effects}
"#,
			pretty(&self.state.world.entities.players.iter().map(|(i, p)| (i, p.position())).collect::<Vec<_>>()),
		)
	}

	// fn draw_error(&self, viewport_size: uvec2) -> SceneGraph {
	// 	SceneGraph::new(viewport_size).with(|sg| sg.bg_color = vec3(0.1, 0.1, 0.3))
	// }

	//-------------------------------------------------------------------------------- text commands

	fn command(&mut self, cmd: &str) -> Result<()> {
		Ok(match &cmd.split_ascii_whitespace().collect::<Vec<_>>()[..] {
			["dbg"] => self.dbg_overlay = !self.dbg_overlay,
			["dbg", arg] => self.dbg_overlay = parse_bool(arg)?,
			["fps"] => self.fps_overlay = !self.fps_overlay,
			["fps", arg] => self.fps_overlay = parse_bool(arg)?,
			_ => self.state.command(cmd)?,
		})
	}
}

// pretty-print a value for debugging.
fn pretty<T: Serialize>(v: &T) -> String {
	ron::ser::to_string_pretty(v, ron::ser::PrettyConfig::new()).unwrap()
}

impl App for Client {
	fn handle_tick(&mut self, inputs: &Inputs) -> StateChange {
		self.tick(inputs)
	}

	fn handle_draw(&self, viewport_size: uvec2) -> SceneGraph {
		self.draw(viewport_size)
	}

	fn handle_command(&mut self, cmd: &str) -> Result<()> {
		self.command(cmd)
	}
}
