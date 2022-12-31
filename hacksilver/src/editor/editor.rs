use super::internal::*;

pub struct Editor {
	ctx: Arc<GraphicsCtx>,

	cfg: BakeOpts,

	camera: Camera,
	mode: Mode,

	assets: AssetsDir,
	map_name: String,

	map: MapData,
	materials: Arc<MaterialPack>,
	zones: ZoneGraph,
	zones_are_baked: bool,

	info: String,
	hud: HUD,

	// where the (ray from the) crosshair intersects the scene.
	crosshair_intersection: Option<CrosshairIntersection>,

	selected_block: Option<Block>,

	cursor_enabled: bool,
	cursor: Cursor,
	recording: Recording,
	history: History,

	// Shows X,Y,Z axes
	axes: Object,
	spawn_marker: Arc<VAO>,
}

enum Mode {
	Normal,
	Baking { cancel: Cancel, done: Receiver<ZoneGraph> },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct CrosshairIntersection {
	// voxel right before where the
	front_voxel: ivec3,
	normal: ivec3,
}

impl Editor {
	/// Create and save a new, mostly empty, map.
	pub fn create(map_name: &str) -> Result<()> {
		let assets = AssetsDir::find()?;
		let map_dir = assets.map_dir(map_name);
		if map_dir.exists() {
			return Err(anyhow!("create new map '{map_name}' map already exists"));
		}
		map_dir.mkdir()?;

		let mut map = MapData::default();
		for (x, z) in cross(0..4, 0..4) {
			map.push(Block {
				pos: ivec3(x, -1, z) * 64,
				rotation: Rotation::UNIT,
				size: Vector3([64, 64, 64]),
				typ: BlockTyp(0),
				mat: MatID(0),
			});
		}
		map.save(&map_dir)
	}

	pub fn load(ctx: &Arc<GraphicsCtx>, map_name: &str) -> Result<Self> {
		let assets = AssetsDir::find()?;
		let cfg = BakeOpts::default();
		let map = MapData::load(&assets.map_dir(map_name))?;
		let materials = Arc::new(MaterialPack::new(ctx, assets.clone())?);

		let zones = ZoneGraph::build_for_editor(ctx, &materials, &map);

		let camera = Camera::default();
		let axes = Object::new(
			&Arc::new(upload_wavefront(ctx, &assets, "axes")?),
			ctx.shader_pack.flat(&upload_image(ctx, &assets, "rainbow", &default())?),
		);
		let hud = HUD::new(ctx);

		let spawn_marker = Arc::new(upload_wavefront(ctx, &assets, "froghead")?);

		Ok(Self {
			ctx: ctx.clone(),
			camera,
			axes,

			mode: Mode::Normal,
			map,
			materials,
			zones,
			zones_are_baked: false,

			assets,
			map_name: map_name.into(),

			cfg,

			info: String::new(),
			hud,

			crosshair_intersection: None,
			selected_block: None,
			cursor: Cursor::new(ctx),
			cursor_enabled: false,
			recording: default(),
			history: default(),

			spawn_marker,
		})
	}

	//================================================================================ TICK

	fn tick(&mut self, inputs: &Inputs) -> StateChange {
		self.info.clear();

		// camera
		self.rotate_camera(inputs);
		self.translate_camera(inputs);

		let state_change = match &self.mode {
			Mode::Normal => self.tick_normal(inputs),
			Mode::Baking { .. } => self.tick_baking(inputs),
		};

		// HUD
		self.hud.show_info(&self.info);
		self.hud.tick(inputs.dt());

		state_change
	}

	fn tick_normal(&mut self, inputs: &Inputs) -> StateChange {
		if !self.cursor_enabled {
			if inputs.is_pressed(Button::ESC) {
				return StateChange::ReleaseCursor;
			}
			if inputs.is_pressed(Button::MOUSE1) || inputs.is_pressed(Button::MOUSE2) {
				self.cursor_enabled = true;
			}
			return StateChange::None;
		}

		self.update_crosshair_intersection();
		self.update_cursor_size(inputs);
		self.update_cursor_pos();
		self.update_selection();
		self.update_cursor_orientation(inputs);
		self.handle_grab(inputs);
		self.handle_click(inputs);
		self.handle_palette(inputs);

		// ESC
		if inputs.is_pressed(Button::ESC) {
			self.cursor_enabled = false;
		}
		StateChange::None
	}

	fn rotate_camera(&mut self, inputs: &Inputs) {
		let mouse_sens = 0.003; // TODO: dt
		self.camera.orientation.yaw = wrap_angle(self.camera.orientation.yaw - inputs.mouse_delta().x() * mouse_sens);
		self.camera.orientation.pitch = (self.camera.orientation.pitch + inputs.mouse_delta().y() * mouse_sens).clamp(-89.0 * DEG, 89.0 * DEG);
	}

	fn translate_camera(&mut self, inputs: &Inputs) {
		const BLOCKS_PER_SEC: f32 = 32.0;
		let speed = BLOCKS_PER_SEC * inputs.dt();
		if inputs.is_down(inputs.FORWARD) {
			self.camera.position += speed * self.camera.orientation.look_dir_h()
		}
		if inputs.is_down(inputs.BACKWARD) {
			self.camera.position -= speed * self.camera.orientation.look_dir_h()
		}
		if inputs.is_down(inputs.LEFT) {
			self.camera.position -= speed * self.camera.orientation.look_right()
		}
		if inputs.is_down(inputs.RIGHT) {
			self.camera.position += speed * self.camera.orientation.look_right()
		}
		if inputs.is_down(inputs.CROUCH) {
			self.camera.position[1] -= speed
		}
		if inputs.is_down(inputs.JUMP) {
			self.camera.position[1] += speed
		}
	}

	fn update_crosshair_intersection(&mut self) {
		// Intersect line-of-sight with scene
		let ray = self.camera.crosshair_ray();
		let hr = self.intersect_block(&ray);
		if hr.attrib.is_none() {
			self.crosshair_intersection = None;
			return;
		}

		// Determine voxel right in front of intersection point
		//
		//                                ||
		//                            +---+|
		//  crosshair ----------------|-->*| intersection point
		//                      voxel +---+|
		//                                || scene wall
		//
		let hitpoint = ray.at(hr.t - 0.01).to_f32();
		let voxel_pos = hitpoint.map(|v| v.floor() as i32);

		// Determine axis-aligned normal vector (towards camera) at intersection point.
		// Found by checking which voxel face is closed to the intersection point (*).
		//  +---------+
		//  |         |
		//  |    +    |
		//  |        *|---> normal
		//  +---------+
		let voxel_center = voxel_pos.to_f32() + vec3(0.5, 0.5, 0.5);
		let delta = hitpoint - voxel_center;
		let axis = delta.map(|v| v.abs()).argmax();
		let dir = -delta[axis].signum() as i32;
		let normal = Matrix3::UNIT[axis] * dir;
		debug_assert!(normal.map(|v| v as f64).is_normalized());

		let crosshair_intersection = Some(CrosshairIntersection { front_voxel: voxel_pos, normal });

		self.crosshair_intersection = crosshair_intersection;
	}

	fn update_cursor_size(&mut self, inputs: &Inputs) {
		let (x, y, z) = self.cursor.size().into();
		self.info.push_str(&format!("cursor size {x} x {y} x {z} (align to {})\n", self.cursor.align()));

		if inputs.mouse_wheel_delta() == 0 {
			return;
		}

		let modifier = (inputs.is_down(Button::SHIFT), inputs.is_down(Button::CONTROL), inputs.is_down(Button::ALT));
		let mask = match modifier {
			// just the scroll wheel: scale size in all directions
			(false, false, false) => ivec3(1, 1, 1),
			// shift: scale vertically
			(true, false, false) => ivec3(0, 1, 0),
			// ctrl: scale left-right (in viewing direction)
			(false, true, false) => nearest_axis_2d(self.camera.orientation.look_right()),
			// alt: scale back-front (in viewing direction)
			(false, false, true) => nearest_axis_2d(self.camera.orientation.look_dir_h()),
			// ctrl+alt: scale horizontally (left-right and back-front, but not up-down)
			(false, true, true) => ivec3(1, 0, 1),
			// your modifiers are too powerful for me
			_ => ivec3(0, 0, 0),
		};
		let delta = mask * inputs.mouse_wheel_delta(); // * self.cursor.align() as i32;
		self.cursor.change_scale(delta);
	}

	fn update_cursor_pos(&mut self) {
		// * the cursor block(s) touch the scene at the crosshair intersection point.
		// * the cursor block(s) are always right in front of the scene, not behind (even partially).
		// * is aligned in the normal plane.
		//
		//                            align
		//                              ^
		//                              |  |
		//                            +---+|
		//  crosshair --------------->|    |
		//                     cursor +---+|
		//                              |  |
		//                              v  | scene wall
		//                           align |
		let new_pos = self.crosshair_intersection.map(|inters| {
			let normal = inters.normal;
			let mut pos = inters.front_voxel;
			let axis = normal.map(|v| v.abs()).argmax();
			let cursor_size = self.cursor.size();
			// ensure cursor is in front of wall
			if normal[axis] < 0 {
				pos[axis] -= cursor_size[axis] as i32 - 1;
			}

			for i in 0..3 {
				if i != axis {
					pos[i] = align_to(pos[i], self.cursor.align());
				}
			}
			pos
		});
		self.cursor.set_pos(new_pos);
	}

	fn update_selection(&mut self) {
		let ray = self.camera.crosshair_ray();
		let hr = self.intersect_block(&ray);
		let blk = hr.attrib;
		self.selected_block = match blk {
			// don't select block if camera is inside it.
			Some(blk) if !blk.bounds32().contains(self.camera.position) => Some(blk),
			_ => None,
		};
	}

	fn handle_grab(&mut self, inputs: &Inputs) {
		if inputs.is_pressed(Button::GRAB) && !inputs.is_down(Button::CONTROL) {
			if let Some(blk) = self.selected_block {
				self.cursor.grab1(blk)
			}
		}

		//if inputs.is_pressed(Button::GRAB) && inputs.is_down(Button::CONTROL) {
		//	if let Some(blk) = self.selected_block {
		//		self.cursor.add(blk)
		//	}
		//}
	}

	fn update_cursor_orientation(&mut self, inputs: &Inputs) {
		match inputs.pressed_button() {
			Some(Button::ROTATE) => self.cursor.rotate(Rotation::ROTY90),
			Some(Button::ROTATE2) => self.cursor.rotate(Rotation::ROTX90),
			_ => (),
		}
	}

	fn handle_click(&mut self, inputs: &Inputs) {
		for button in inputs.buttons_pressed() {
			match button {
				Button::MOUSE1 => self.handle_l_click(),
				Button::MOUSE2 => self.handle_r_click(),
				Button::PAINT => self.history.commit_change(), // start recording paint
				_ => (),
			}
		}

		for button in inputs.buttons_down() {
			match button {
				Button::PAINT => self.handle_paint_no_commit(),
				_ => (),
			}
		}

		for button in inputs.buttons_released() {
			match button {
				Button::PAINT => self.history.commit_change(),
				_ => (),
			}
		}
	}

	fn handle_palette(&mut self, inputs: &Inputs) {
		if inputs.is_down(Button::Key(VirtualKeyCode::V)) && inputs.mouse_wheel_delta() != 0 {
			let mat = match self.selected_block {
				Some(block) => block.mat,
				None => self.cursor.material(),
			};
			let empty = String::new();
			let current = self.map.palette.material_name_for(mat).unwrap_or(&empty);
			let next = self.materials.next_after(current, inputs.mouse_wheel_delta());
			self.map.palette.set(mat, next);
			println!("material: {next}");
			self.invalidate_all_zones();
			self.ensure_all_zones();
		}
	}

	fn handle_l_click(&mut self) {
		for b in self.cursor.abs_prototype() {
			self.add_block(b);
			self.history.record_add(b);
		}
		self.history.commit_change();
		self.ensure_all_zones();
	}

	fn handle_r_click(&mut self) {
		if let Some(blk) = self.selected_block {
			self.remove_block(&blk);
			self.history.record_remove(&blk);
		}
		self.history.commit_change();
		self.ensure_all_zones();
	}

	fn handle_paint_no_commit(&mut self) {
		if let Some(blk) = self.selected_block {
			let mat = self.cursor.material();
			// only make a change if needed
			// (we can continuously paint by holding the mouse,
			// don't want to record no-op changes at 60 per second).
			if blk.mat != mat {
				self.remove_block(&blk);
				self.history.record_remove(&blk);
				let blk = blk.with(|b| b.mat = mat);
				self.add_block(blk);
				self.history.record_add(blk);
				self.ensure_all_zones();
			}
		}
	}

	fn undo(&mut self) {
		let change = self.history.undo();
		self.apply_change(change);
		self.ensure_all_zones();
	}

	fn redo(&mut self) {
		let change = self.history.redo();
		self.apply_change(change);
		self.ensure_all_zones();
	}

	fn apply_change(&mut self, change: Change) {
		for add in change.add {
			self.add_block(add)
		}
		for rm in change.remove {
			self.remove_block(&rm)
		}
	}

	fn rebuild(&mut self) {
		self.zones.clear();
		self.ensure_all_zones();
	}

	//-------------------------------------------------------------------------------- mutations

	fn remove_block(&mut self, b: &Block) {
		let _ = self.map.remove(b);
		self.invalidate_zones_for(b.ibounds());
		self.recording.record_remove(b);
	}

	fn add_block(&mut self, b: Block) {
		self.map.push(b);
		self.invalidate_zones_for(b.ibounds());
		self.recording.record_add(b);
	}

	fn invalidate_zones_for(&mut self, open_range: BoundingBox<i32>) {
		if self.zones_are_baked {
			// Baking optimizes faces and may zone them differently from the editable view.
			// So rebuild everything when we edit after bake.
			self.invalidate_all_zones();
		} else {
			for pos in open_range.vertices() {
				let pos = trunc_to_zone(pos);
				// invalidate a 3x3x3 zone cube around each vertex.
				// this is almost certainly more than necessary.
				for (dx, dy, dz) in cross3(-1..=1, -1..=1, -1..=1) {
					let pos = pos + ivec3(dx, dy, dz) * ZONE_ISIZE;
					self.zones.remove(pos);
				}
			}
		}
	}

	fn invalidate_all_zones(&mut self) {
		self.zones.clear();
		self.zones_are_baked = false;
	}

	// Ensure that all zones are ready for rendering.
	// Build zones that are not yet ready.
	fn ensure_all_zones(&mut self) {
		for &zpos in self.map.blocks_by_zone.keys() {
			debug_assert!(is_zone_aligned(zpos));
			if !self.zones.contains_key(zpos) {
				if let Some(blocks) = self.map.blocks_by_zone.get(&zpos) {
					let zone = Zone::build_for_editor(&self.ctx, &self.materials, &self.map, blocks.iter().copied());
					self.zones.insert(zpos, zone);
				}
			}
		}
	}

	//-------------------------------------------------------------------------------- accessors

	fn intersect_block(&self, ray: &Ray<f64>) -> HitRecord<f64, Block> {
		// TODO: accelerate
		let mut hr = HitRecord::new();
		for block in self.map.blocks_as_ref() {
			if let Some(t) = block.intersect(ray) {
				hr.record(t, block)
			}
		}
		hr
	}

	fn map_dir(&self) -> MapDir {
		self.assets.map_dir(&self.map_name)
	}

	//-------------------------------------------------------------------------------- text commands

	fn command(&mut self, cmd: &str) -> Result<()> {
		Ok(match &cmd.split_ascii_whitespace().collect::<Vec<_>>()[..] {
			&["prod"] => self.bake_prod()?,
			&["bake"] => self.start_baking(),
			&["rebuild"] => self.rebuild(),
			&["lm", "res", arg] => self.cfg.lightmap_resolution = arg.parse()?,
			&["lm", "filter", arg] => self.cfg.lightmap_filter_radius = arg.parse()?,
			&["lm", "refl", arg] => self.cfg.lightmap_reflectivity = arg.parse()?,
			&["lm", "lo" | "low"] => self.lm_qual_cmd(&BakeOpts::LOW_QUALITY),
			&["lm", "med" | "medium"] => self.lm_qual_cmd(&BakeOpts::MEDIUM_QUALITY),
			&["lm", "hi" | "high"] => self.lm_qual_cmd(&BakeOpts::HIGH_QUALITY),
			&["lm", "sun", "samples" | "s", arg] => self.cfg.lightmap_lamps_samples = arg.parse()?,
			&["lm", "sky", "samples" | "s", arg] => self.cfg.lightmap_sky_samples = arg.parse()?,
			&["lm", "indirect", "samples" | "s", arg] => self.cfg.lightmap_indirect_samples = arg.parse()?,
			//&["lm", "depth", arg] => self.cfg.lightmap_indirect_depth = arg.parse()?,
			&["lm", "smudge", arg] => self.cfg.lightmap_smudge = parse_bool(arg)?,
			//&["lm", "vis", "only"] => self.cfg.lightmap_visibility_only = true,
			&["lm", "sun", "only"] => self.cfg.lightmap_sun_only = true,
			&["lm", "sky", "only"] => self.cfg.lightmap_sky_only = true,
			&["lm", "em" | "emissive", "only"] => self.cfg.lightmap_emission_only = true,
			&["lm", "in" | "scattered", "only"] => self.cfg.lightmap_scattered_only = true,
			&["lm", "amb" | "ambient", "only"] => self.cfg.lightmap_ambient_only = true,
			&["lm", "blur", "all", arg] => self.cfg.lightmap_blur_all = arg.parse()?,
			&["lm", "blur", "sun", arg] => self.cfg.lightmap_blur_sun = arg.parse()?,
			//&["lm", "direct", "only"] => ok(self.cfg.lightmap_direct_only = true),
			&["lm", "validity"] => self.cfg.lightmap_show_validity = true,
			&["lm", "outline", arg] => self.cfg.lightmap_outline = parse_bool(arg)?,
			&["lm", "nearest"] => self.cfg.lightmap_nearest = true,
			&["lm", "offset", arg] => self.cfg.lightmap_offset = arg.parse()?,
			&["lm", "nearest", arg] => self.cfg.lightmap_nearest = parse_bool(arg)?,
			&["lm", "stitch", arg] => self.cfg.lightmap_stitch = parse_bool(arg)?,
			//&["tex", arg] => self.cfg.disable_textures = !arg.parse()?,
			&["bt" | "blocktype", arg] => self.cursor.set_blocktyp(arg.parse()?),
			&["m" | "mat" | "material", arg] => self.cursor.set_material(MatID(arg.parse()?)),
			&["sun", "dir"] => LOG.write(format!("sun dir: {}", self.map.meta.sun_dir)),
			&["sun", "dir", x, y, z] => self.map.meta.sun_dir = parse_vec(x, y, z)?.normalized(),
			&["sun", "color", x, y, z] => self.map.meta.sun_color = parse_vec(x, y, z)?,
			&["sky", "color", x, y, z] => self.map.meta.sky_color = parse_vec(x, y, z)?,
			&["save"] => self.save()?,
			&["a" | "align", arg] => self.cursor.set_linear_align(arg.parse()?),
			&["r" | "rec", arg] => self.start_recording(arg),
			&["r" | "rec"] => self.stop_recording()?,
			&["u" | "use", arg] => self.use_recording(arg)?,
			//&["x" | "cut"] => ok(self.recording.start_cut()),
			//&["p" | "paste"] => ok(self.cursor.set_prototype(self.recording.paste())),
			//&["p" | "paste", arg] => ok(self.recording.record_clipboard(arg)),
			&["undo"] => self.undo(),
			&["redo"] => self.redo(),
			&["spawn"] => self.add_spawn_point(),
			&["rmspawn"] => self.rm_spawn_point(),

			//&["print", "md"] => Ok(format!(
			//	"sun_dir: {}\nsun_color: {}\nsky_color:{}",
			//	self.map.data.sun_dir, self.map.data.sun_color, self.map.data.sky_color
			//)),
			_ => return Err(anyhow!("unknown command: '{cmd}'")),
		})
	}

	fn add_spawn_point(&mut self) {
		// TODO: orientation is wrong. OR is the shader wrong? Test against axes, XYZ blocks
		let dir = nearest_axis_2d(self.camera.orientation.look_dir()).to_f32();
		let yaw = f32::atan2(dir.x(), dir.z());
		if let Some(CrosshairIntersection { front_voxel, .. }) = self.crosshair_intersection {
			let spawn_point = SpawnPoint { pos: front_voxel, yaw };
			LOG.write(format!(
				"adding spawn point #{} @{}, yaw: {} deg",
				self.map.meta.spawn_points.len() + 1,
				spawn_point.position(),
				spawn_point.yaw / DEG
			));
			self.map.meta.spawn_points.push(spawn_point);
		}
	}

	fn rm_spawn_point(&mut self) {
		self.map.meta.spawn_points.pop();
	}

	fn start_recording(&mut self, name: &str) {
		self.recording.start_recording(self.map_dir(), name)
	}

	fn stop_recording(&mut self) -> Result<()> {
		let name = self.recording.stop_recording()?;
		// immediately use recording on stop.
		self.use_recording(&name)
	}

	fn use_recording(&mut self, name: &str) -> Result<()> {
		let blocks = self.recording.get(&self.map_dir(), name)?;
		self.cursor.set_prototype(blocks);
		Ok(())
	}

	fn lm_qual_cmd(&mut self, cfg: &BakeOpts) {
		self.cfg = cfg.clone();
	}

	fn save(&self) -> Result<()> {
		self.map.save(&self.assets.map_dir(&self.map_name))
	}

	//-------------------------------------------------------------------------------- bake

	// Do a production (high quality) lightmap bake and save it.
	fn bake_prod(&mut self) -> Result<()> {
		// bake & save
		let hzones = HZoneGraph::bake(&self.cfg, &self.materials, &self.map, Cancel::new());
		hzones.save(&self.map_dir())?;

		// show the baked zones
		self.zones = ZoneGraph::upload(&self.ctx, &self.materials, &self.map.palette, hzones);
		self.zones_are_baked = true; //

		Ok(())
	}

	/// Start baking lightmap in the background.
	/// `mode` becomes `Baking`, which locks any edits to the map until baking is done
	/// (or canceled).
	fn start_baking(&mut self) {
		let (send, recv) = mpsc::channel();
		let cancel = Cancel::new();

		self.cursor_enabled = false;
		self.mode = Mode::Baking { done: recv, cancel: cancel.clone() };
		self.materials.clear_cache(); // free up some graphics memory

		{
			let ctx = self.ctx.clone();
			let cfg = self.cfg.clone();
			let materials = self.materials.clone();
			let map = self.map.clone();
			let _ = thread::spawn(move || {
				let zonegraph = ZoneGraph::build_baked(&ctx, &cfg, &materials, &map, cancel);
				send.send(zonegraph).unwrap();
			});
		}
	}

	fn tick_baking(&mut self, inputs: &Inputs) -> StateChange {
		self.info.push_str("baking...\n");
		let (cancel, done) = match &self.mode {
			Mode::Baking { cancel, done } => (cancel, done),
			_ => unreachable!(),
		};

		// cancel baking on `ESC`.
		if inputs.is_pressed(Button::ESC) {
			cancel.cancel();
			self.mode = Mode::Normal;
			return StateChange::None;
		}

		use mpsc::TryRecvError::*;
		match done.try_recv() {
			Ok(zones) => self.finish_baking(zones),
			Err(Disconnected) => {
				eprintln!("ERROR: bake: recv: sender disconnected");
				self.mode = Mode::Normal;
			}
			Err(Empty) => (),
		}

		StateChange::None
	}

	fn finish_baking(&mut self, zones: ZoneGraph) {
		self.zones = zones;
		self.zones_are_baked = true;
		self.mode = Mode::Normal;
	}

	//================================================================================ DRAW

	fn draw(&self, viewport: uvec2) -> SceneGraph {
		let mut sg = SceneGraph::new(viewport).with(|sg| {
			sg.bg_color = self.map.meta.sky_color;
			sg.camera = self.camera.clone()
		});

		self.zones.draw_on(&mut sg);
		sg.push(self.axes.clone());

		// Block selection after map (draws over map)
		if self.cursor_enabled {
			sg.objects.extend_from_slice(&self.draw_selection());
			sg.objects.extend_from_slice(&self.cursor.draw());
		}

		for p in &self.map.meta.spawn_points {
			let transform = translation_matrix(p.position()) * p.orientation().matrix() * scale_matrix(8.0);
			sg.push(Object::new(&self.spawn_marker, self.ctx.shader_pack.entity(&self.ctx.fallback_texture, transform)));
		}

		// ! Crosshair text drawn last to be on top.
		self.hud.draw_on(&mut sg);

		sg
	}

	fn draw_selection(&self) -> SmallVec<[Object; 2]> {
		let ctx = &self.ctx;
		match self.selected_block {
			None => smallvec![],
			Some(blk) => smallvec![Object::new(
				&Arc::new(ctx.upload_meshbuffer(&Self::block_meshbuffer(&blk))),
				ctx.shader_pack.highlight(&ctx.fallback_texture),
			)],
		}
	}

	fn block_meshbuffer(blk: &Block) -> MeshBuffer {
		let mut buf = MeshBuffer::new();
		for face in blk.faces() {
			buf.append(&face_meshbuffer(&face, default(), default(), default()));
		}
		buf
	}
}

impl App for Editor {
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

fn parse_vec(x: &str, y: &str, z: &str) -> Result<vec3> {
	Ok(vec3(x.parse()?, y.parse()?, z.parse()?))
}

// In each direction, the cursor range gets aligned down the same power of two as present in the size.
// E.g.:
// 	size 1 => align 1
// 	size 2 => align 2
// 	size 3 => align 1
// 	size 4 => align 4
// 	size 5 => align 5
// 	size 6 => align 2
// 	size 8 => align 8
//  ...
//fn aligned_cursor_pos(pos: ivec3, size: Vector3<u8>) -> ivec3 {
//	let size = size.convert();
//	pos.zip(size, |pos, size| pos & !(2i32.pow(size.trailing_zeros()) - 1))
//}

fn align_to(v: i32, align: u8) -> i32 {
	v & !(2i32.pow(align.trailing_zeros()) - 1)
}

fn nearest_axis_2d(dir: vec3) -> ivec3 {
	[ivec3::EX, ivec3::EZ, -ivec3::EX, -ivec3::EZ]
		.into_iter()
		.map(|v| (v, v.to_f32().dot(dir)))
		.max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
		.unwrap()
		.0
}
