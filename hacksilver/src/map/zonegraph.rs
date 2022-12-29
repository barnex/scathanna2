use super::internal::*;

/// The `ZoneGraph` splits a Map into large chunks (`Zone`s) that are drawn independently.
///
/// At runtime, large swaths of the `ZoneGraph` can be culled e.g. when they are entirely
/// outside of the camera frustum or too far away.
///
/// E.g.: in the diagram below, player `p` looks to the right. The zone containing `Object`s
/// D, E, F will be drawn, but not the `Zone` containing A, B.
///
///    +------+------+------+
///    | A    |      |      |
///    |    B |  p-> | D EF |
///    +------+------+------+
///    |      | C    |      |
///    |      |      |      |
///    +------+------+------+
///
#[derive(Default)]
pub struct ZoneGraph {
	pub zones: HashMap<ivec3, Zone>,
}

/// The host equivalent of `ZoneGraph`, for Serialization and staging purposes.
#[derive(Default)]
pub struct HZoneGraph {
	pub hzones: HashMap<ivec3, HZone>,
}

impl HZoneGraph {
	pub fn bake(cfg: &BakeOpts, materials: &Arc<MaterialPack>, mapdata: &MapData, cancel: Cancel) -> Self {
		// 1) splinch blocks to faces.
		let faces = mapdata.blocks().map(|b| b.faces()).flatten().collect::<Vec<_>>();

		// 1b) optimize faces, remove fully occluded ones.
		let faces = optimize_faces(faces);

		// 2) bake faces to corresponding images (parallel array).
		let snips = bake_raytraced(&cfg, materials, mapdata, &faces, cancel);

		// 3) group snippets by zone
		let snips_by_zone = group_by(snips, |snip| zone_for(&snip.face));

		// 4) assemble snippets into Zones (meshes+textures+lightmaps on the GPU).
		let hzones = snips_by_zone.map_values(|snips| HZone::build(snips));

		Self { hzones }
	}

	pub fn load(map_dir: &MapDir) -> Result<Self> {
		let hobjs_by_zone = Self::load_mesh(&map_dir)?;

		let mut hzones = HashMap::default();
		for (zpos, objs) in hobjs_by_zone {
			let (indirect_atlas, visibility_atlas) = Self::load_lm(&map_dir, zpos)?;
			hzones
				.insert(
					zpos,
					HZone {
						objs,
						indirect_atlas,
						visibility_atlas,
					},
				)
				.and_then::<(), _>(|_| panic!("BUG: duplicate key?"));
		}

		Ok(HZoneGraph { hzones })
	}

	fn load_mesh(map_dir: &MapDir) -> Result<HashMap<ivec3, Vec<HObject>>> {
		load_bincode_gz(&map_dir.mesh_file())
	}

	fn load_lm(map_dir: &MapDir, zpos: ivec3) -> Result<(RgbImage, RgbImage)> {
		let lm_dir = &map_dir.lightmap_dir();
		let indirect_atlas = image::open(&lm_dir.join(Self::lm_name(INDIRECT_LM, zpos)))?.into_rgb8();
		let visibility_atlas = image::open(&lm_dir.join(Self::lm_name(VISIBILITY_LM, zpos)))?.into_rgb8();
		Ok((indirect_atlas, visibility_atlas))
	}

	pub fn save(&self, map_dir: &MapDir) -> Result<()> {
		self.save_lightmaps(map_dir)?;
		self.save_mesh(map_dir)?;
		Ok(())
	}

	fn save_mesh(&self, map_dir: &MapDir) -> Result<()> {
		let mesh_by_zone = self.hzones.iter().map(|(zpos, hzone)| (*zpos, &hzone.objs)).collect::<HashMap<_, _>>();
		save_bincode_gz(&mesh_by_zone, &map_dir.mesh_file())
	}

	fn save_lightmaps(&self, map_dir: &MapDir) -> Result<()> {
		let lm_dir = map_dir.lightmap_dir();
		mkdir(&lm_dir).unwrap_or_else(|err| LOG.write(format!("ERROR: mkdir {:?}: {}", &lm_dir, err)));
		LOG.write("saving lightmaps");

		for (&zpos, hzone) in &self.hzones {
			Self::save_lm(&lm_dir, INDIRECT_LM, &hzone.indirect_atlas, zpos)?;
			Self::save_lm(&lm_dir, VISIBILITY_LM, &hzone.visibility_atlas, zpos)?;
		}
		LOG.replace_last_line("saving lightmaps: done");
		Ok(())
	}

	fn save_lm(dir: &Path, prefix: &str, img: &RgbImage, zpos: ivec3) -> Result<()> {
		let fname = dir.join(Self::lm_name(prefix, zpos));
		LOG.replace_last_line(format!("saving {fname:?}"));
		let f = File::create(fname)?;
		let mut b = BufWriter::new(f);
		Ok(img.write_to(&mut b, image::ImageOutputFormat::Png)?)
	}

	fn lm_name(prefix: &str, zpos: ivec3) -> String {
		let apos = zpos.map(|v| (v + (u16::MAX / 2) as i32) as u16);
		format!("{prefix}{:04x}{:04x}{:04x}.png", apos.x(), apos.y(), apos.z())
	}
}

impl ZoneGraph {
	pub fn draw_on(&self, sg: &mut SceneGraph) {
		sg.objects.extend(
			self.zones.values().map(|zone| &zone.objs).flatten().cloned(), // Arc clones
		)
	}

	pub fn clear(&mut self) {
		self.zones.clear()
	}

	pub fn remove(&mut self, zpos: ivec3) {
		debug_assert!(is_zone_aligned(zpos));
		self.zones.remove(&zpos);
	}

	pub fn contains_key(&self, zpos: ivec3) -> bool {
		self.zones.contains_key(&zpos)
	}

	pub fn insert(&mut self, zpos: ivec3, zone: Zone) {
		self.zones.insert(zpos, zone);
	}

	// Turn a BlockList into a renderable mesh divided into zones (indexed by zoned position).
	pub fn build_baked(ctx: &GraphicsCtx, cfg: &BakeOpts, materials: &Arc<MaterialPack>, mapdata: &MapData, cancel: Cancel) -> Self {
		let hzonegraph = HZoneGraph::bake(cfg, materials, mapdata, cancel);
		Self::upload(ctx, materials, &mapdata.palette, hzonegraph)
	}

	pub fn upload(ctx: &GraphicsCtx, materials: &Arc<MaterialPack>, palette: &Palette, hzonegraph: HZoneGraph) -> Self {
		Self {
			zones: hzonegraph
				.hzones //
				.map_values(|hzone| Zone::upload(ctx, materials, palette, &hzone)),
		}
	}

	pub fn build_for_editor(ctx: &GraphicsCtx, materials: &MaterialPack, map: &MapData) -> Self {
		let blocks_by_zone = group_by(map.blocks(), |b| zone_for(b));
		Self {
			zones: blocks_by_zone //
				.map_values(|blocks| Zone::build_for_editor(ctx, materials, map, blocks.into_iter())),
		}
	}
}

pub struct Zone {
	pub objs: Vec<Object>,
}

// ================================================================================ Game

pub struct HZone {
	pub objs: Vec<HObject>,
	indirect_atlas: RgbImage,
	visibility_atlas: RgbImage,
}

impl HZone {
	// After baking faces to snippets (face + baked light image),
	// Assemble the lightmap snippets into a lightmap atlas,
	// and assemble the individual faces into a mesh geometry (one mesh per material).
	//
	//    loose faces   =>    one meshbuffer per material
	//
	//     +---+
	//    / A /    +            +---+              +
	//   +---+    /|           / A /              /|
	//   +---+   +B+    =>    +---+          +---+B+
	//   | B |   | /                         | B |/
	//   +---+   +                           +---+
	//
	fn build(snips: Vec<Snippet2>) -> Self {
		// 1) Assemble lightmap snippets (per-face images) into a per-zone Atlas.
		// All triangles in a Zone share the same lightmap Atlas.
		let (atlas_size, positions) = lm_alloc_snips(&snips);

		let indirect_atlas = RgbImage::new(atlas_size.x(), atlas_size.y()) //
			.with(|img| lm_copy_snips(img, snips.iter().map(|snip| &snip.ambient), &positions));

		let visibility_atlas = RgbImage::new(atlas_size.x(), atlas_size.y()) //
			.with(|img| lm_copy_snips(img, snips.iter().map(|snip| &snip.direct_visibility), &positions));

		// 2) Build one vertex array per material
		// (inside a zone, each material will get one draw call).
		let mut meshbuf_by_mat = HashMap::<MatID, MeshBuffer>::default();
		for (snip, lm_offset) in snips.iter().zip(positions) {
			let face_buf = face_meshbuffer(&snip.face, lm_offset, snip.dimensions(), atlas_size);
			meshbuf_by_mat.entry(snip.face.mat).or_default().append(&face_buf)
		}

		let mut objs = vec![];
		for (mat_id, meshbuf) in meshbuf_by_mat {
			objs.push(HObject { meshbuf, mat_id });
		}

		Self {
			objs,
			indirect_atlas,
			visibility_atlas,
		}
	}
}

impl Zone {
	// Assemble and upload textures, vertex arrays, etc. needed to draw a single `Zone`.
	// fn build_baked(ctx: &GraphicsCtx, cfg: &EditorConfig, materials: &MaterialPack, palette: &Palette, snips: Vec<Snippet>) -> Self {
	// 	Self::upload(ctx, cfg, materials, palette, &HZone::build(snips))
	// }

	// Upload a host Zone to the GPU.

	fn upload(ctx: &GraphicsCtx, materials: &MaterialPack, palette: &Palette, hzone: &HZone) -> Self {
		//let filter = match ctx.opts.trilinear_enabled() {
		//	true => LINEAR,
		//	false => NEAREST,
		//};

		// TODO: avoid clones -- get rid of image::Image
		let indirect_atlas = Arc::new(ctx.upload_image_nomip(&hzone.indirect_atlas.clone().into(), &ctx.opts.lightmap_filter()));
		let visibility_atlas = Arc::new(ctx.upload_image_nomip(&hzone.visibility_atlas.clone().into(), &default()));

		let material_for = |materials, mat_id| match ctx.opts.textures_enabled() {
			true => palette.material_for(materials, mat_id),
			false => GMaterial::uniform(ctx, vec3::ONES),
		};

		// 3) Upload vertex arrays to GPU
		let mut objs = vec![];
		for hobj in &hzone.objs {
			let vao = Arc::new(ctx.upload_meshbuffer(&hobj.meshbuf));
			let material = material_for(materials, hobj.mat_id);
			let mat = ctx.shader_pack.normalmap(
				&material.diffuse,
				&indirect_atlas,
				&material.normal.unwrap_or_else(|| Arc::new(uniform_texture(ctx, vec4(0.0, 0.0, 1.0, 0.0)))),
				&visibility_atlas,
			);
			//let mat = Shader::lightmap(ctx, &material.diffuse, &indirect_atlas);
			objs.push(Object::new(&vao, mat));
		}

		Self { objs }
	}

	pub fn build_for_editor(ctx: &GraphicsCtx, materials: &MaterialPack, map: &MapData, blocks: impl Iterator<Item = Block>) -> Self {
		let faces = blocks.map(|b| b.faces()).flatten().collect::<Vec<_>>();
		let faces = optimize_faces(faces);
		// Not used by editor shader
		let lm_offset = default();
		let img_size = default();
		let atlas_size = default();

		// 2) Build one vertex array per material
		let mut objs = vec![];

		// (inside a zone, each material will get one draw call).
		let mut meshbuf_by_mat = HashMap::<MatID, MeshBuffer>::default();
		for face in faces.iter() {
			let face_buf = face_meshbuffer(&face, lm_offset, img_size, atlas_size);
			meshbuf_by_mat.entry(face.mat).or_default().append(&face_buf)
		}

		for (mat, meshbuf) in meshbuf_by_mat {
			let vao = Arc::new(ctx.upload_meshbuffer(&meshbuf));

			let material = map.palette.material_for(materials, mat);
			let mat = //if ctx.opts.wireframe {
			//	ctx.shader_pack.lines(ctx, &ctx.fallback_texture)
			//} else {
				ctx.shader_pack.editor(&material.diffuse);
			//};

			objs.push(Object::new(&vao, mat));
		}

		Self { objs }
	}
}

//================================================================================  Editor

pub fn optimize_faces(faces: Vec<Face>) -> Vec<Face> {
	// 1) Remove identical, touching faces.
	// E.g. Remove these two overlapping, identical faces (marked 'x'):
	//
	//   +------+------+
	//   |      |      |
	//   |      x      |
	//   |      |      |
	//   +------+------+
	type Key = SmallVec<[ivec3; 4]>;
	let mut key_to_ids = HashMap::<Key, SmallVec<[usize; 2]>>::default();
	for (id, face) in faces.iter().enumerate() {
		let mut key = face.vertices(); // canonical key. same for same-shaped faces.
		key.sort_by_key(|v| (v[0], v[1], v[2]) /*any deterministic vertex order will do*/);
		key_to_ids.entry(key).or_default().push(id);
	}
	let opt = key_to_ids //
		.into_iter()
		.filter(|(_, ids)| ids.len() == 1)
		.map(|(_, ids)| faces[ids[0]].clone())
		.collect::<Vec<_>>();

	// 2) TODO: remove hidden, non-same-size faces

	// 3) TODO: merge adjacent faces if possible

	opt
}

/// Given a Face, and its region (offset + size) in the lightmap atlas,
/// create a corresponding mesh.
///
/// Choosing a vertex's lightmap coordinates (UV position in lightmap atlas):
///
///   * First, each Face's got allocated an rectangular region in the atlas,
///     as determined by the lightmap allocator. This starts at position `offset`.
///   * Second, each region has a 2 pixel margin to avoid neighboring regions "bleeding"
///     into this one because of linear interpolation. (In theory this should not
///     be needed but round-off errors can still cause bleeding,
///     especially visible with MSAA).
///   * Finally, 0.5 pixels need to be added to place the vertex exactly at the
///     center of its corresponding pixel.
///
///    atlas
///    +------------------------------->
///    |               
///    |   offset (start of image)                 
///    |          +-----------------
///    |          |       
///    |          |       margin for bleeding (2pix)
///    |          |      +-----------
///    |          |      |    0.5pix to center vertex inside pixel
///    |          |      |   UV-----
///    |          |      |   |
///    |          |      |   |
///    v
pub fn face_meshbuffer(face: &Face, lm_offset_pix: uvec2, snip_pix: uvec2, lm_atlas_pix: uvec2) -> MeshBuffer {
	let normal = face.normalized_normal();

	let (offx, offy) = lm_offset_pix.to_f32().into();
	let (sizex, sizey) = snip_pix.to_f32().into();
	let atlas_pix = lm_atlas_pix.to_f32();

	// Margin for bleeding + 0.5 pixel offset to center.
	const M: f32 = BorderedImg::SNIPPET_MARGIN as f32 + 0.5;

	let uv_basis = uv_basis(face);
	let [tangent_u, tangent_v] = [uv_basis[0], uv_basis[1]];

	let position = face.origin().to_f32();
	let o = VertexLM {
		position,
		texcoords: uv_project(&uv_basis, position),
		normal,
		lightcoords: vec2(offx + M, offy + M).div2(atlas_pix),
		tangent_u,
		tangent_v,
	};

	let [n1, n2] = face.sized_tangents();

	let position = o.position + n1.to_f32();
	let a = VertexLM {
		position,
		texcoords: uv_project(&uv_basis, position),
		normal,
		lightcoords: vec2(offx + sizex - M, offy + M).div2(atlas_pix),
		tangent_u,
		tangent_v,
	};

	let position = o.position + n2.to_f32();
	let b = VertexLM {
		position,
		texcoords: uv_project(&uv_basis, position),
		normal,
		lightcoords: vec2(offx + M, offy + sizey - M).div2(atlas_pix),
		tangent_u,
		tangent_v,
	};

	let position = o.position + n1.to_f32() + n2.to_f32();
	let c = VertexLM {
		position,
		texcoords: uv_project(&uv_basis, position),
		normal,
		lightcoords: vec2(offx + sizex - M, offy + sizey - M).div2(atlas_pix),
		tangent_u,
		tangent_v,
	};

	match face.shape {
		FaceShape::Rect => MeshBuffer::rect(&[a, o, b, c]),
		FaceShape::Tri => MeshBuffer::triangle(&[a, o, b]),
	}
}

/// Decompose a list of Blocks into a list of Faces.
///   +---+                           +
///  /   /|                          /|
/// +---+ +    =>   +---+    +---+  + +
/// |   |/          |   |   /   /   |/
/// +---+           +---+  +---+    +
pub fn splinch_to_faces(blocks: impl Iterator<Item = Block>) -> impl Iterator<Item = Face> {
	blocks.map(|b| b.faces()).flatten()
}
