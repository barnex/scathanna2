//! Find and load asset files (textures, meshes).
use super::internal::*;

// Path to the `assets/` directory.
// Typestate pattern ensures correct use.
#[derive(Clone)]
pub struct AssetsDir(PathBuf);

// Path to a map directory (e.g. `deck.hx`).
// Typestate pattern ensures correct use.
pub struct MapDir(PathBuf);

impl AssetsDir {
	/// Find the absolute path of the assets directory.
	/// Search in the current working directory and the executable's directory.
	pub fn find() -> Result<Self> {
		const ASSETS: &str = "assets";

		if let Ok(dir) = std::env::current_dir() {
			log::info!("searching for assets in working directory: {}", dir.to_string_lossy());
			let abs = dir.join(ASSETS);
			if abs.exists() {
				return Ok(Self(abs));
			}
		}

		let exe = std::env::current_exe()?;
		if let Some(dir) = exe.parent() {
			log::info!("searching for assets in executable directory: {}", dir.to_string_lossy());
			let abs = dir.join(ASSETS);
			if abs.exists() {
				return Ok(Self(abs));
			}
		}

		Err(anyhow!("assets directory not found.\nBe sure to run this program form a directory that contains 'assets/'."))
	}

	pub fn find_all_maps(&self) -> Result<Vec<String>> {
		let dir = self.maps_dir();
		Ok(read_dir_names(&dir)? //
			.filter_map(|f| f.file_name().map(|n| n.to_owned()))
			.map(|n| n.to_string_lossy().to_string())
			.filter_map(|n| n.strip_suffix(".hx").map(|n| n.to_owned()))
			.collect::<Vec<_>>()
			.with(|maps| maps.sort()))
	}

	// find an `obj` or `obj.gz` file in the assets directory.
	pub fn find_obj(&self, base: &str) -> Result<PathBuf> {
		Self::find_asset(&self.0.join("obj"), base, &["obj", "obj.gz"])
	}

	/// Find absolute path to a texture file with `base` name. E.g.:
	///   "lava" => "/path/to/textures/lava.png"
	fn find_texture(&self, base: &str) -> Result<PathBuf> {
		Self::find_asset(&self.textures_dir(), base, &["png", "jpg", "jpeg"])
	}

	/// Find the absolute path of an asset file. E.g.:
	///   find_asset("/path/to/assets/textures", "lava", &["png", "jpg"])? =>  /path/to/assets/textures/lava.jpg
	fn find_asset(dir: &PathBuf, base: &str, extensions: &[&str]) -> Result<PathBuf> {
		for ext in extensions {
			let file = dir.join(base.to_owned() + "." + ext); // note: do not use .with_extension, *replaces* extension.
			if file.exists() {
				return Ok(file);
			}
		}
		Err(anyhow!("asset not found: {:?} with extension {}", dir.join(base), extensions.join(", ")))
	}

	fn maps_dir(&self) -> PathBuf {
		self.0.join("maps")
	}

	pub fn map_dir(&self, map_name: &str) -> MapDir {
		MapDir(self.dir().join("maps").join(map_name.to_string() + ".hx"))
	}

	pub fn audio_dir(&self) -> PathBuf {
		self.0.join("audio")
	}

	pub fn materials_dir(&self, resolution: u32) -> PathBuf {
		self.0.join("materials").join(resolution.to_string())
	}

	pub fn settings_file(&self, file: &str) -> Result<PathBuf> {
		Ok(self.0.parent().ok_or(anyhow!("assets parent directory not found"))?.join(file))
	}

	// TODO: resolution
	fn textures_dir(&self) -> PathBuf {
		self.0.join("textures")
	}

	fn dir(&self) -> &Path {
		&self.0
	}
}

impl MapDir {
	pub fn blocks_file(&self) -> PathBuf {
		self.0.join("blocks.bincode.gz")
	}

	pub fn mesh_file(&self) -> PathBuf {
		self.0.join("mesh.bincode.gz")
	}

	pub fn metadata_file(&self) -> PathBuf {
		self.0.join("metadata.json")
	}

	pub fn palette_file(&self) -> PathBuf {
		self.0.join("palette.json")
	}

	pub fn lightmap_dir(&self) -> PathBuf {
		self.0.join("lm")
	}

	pub fn recordings(&self) -> PathBuf {
		self.0.join("rec")
	}

	pub fn exists(&self) -> bool {
		self.0.exists()
	}

	pub fn mkdir(&self) -> Result<()> {
		mkdir(&self.0)
	}
}

pub const INDIRECT_LM: &str = "amb";
pub const VISIBILITY_LM: &str = "sun";

// Load a wavefront file (no extension, e.g. "frog") from disk, upload to GPU as Vertex Array.
pub fn upload_wavefront(ctx: &GraphicsCtx, assets: &AssetsDir, base: &str) -> Result<VAO> {
	Ok(ctx.upload_meshbuffer(&load_wavefront_merged(assets, base)?))
}

/// Find and load a wavefront OBJ file by base name (no extension, e.g. "rocket").
/// Searches `{assets}/obj` for `{base}.obj`, `{base.obj.gz}`.
/// All Objects (in the wavefront sense, e.g. 'Cube.001', 'Cube.002')  are merged into one.
/// Not cached.
pub fn load_wavefront_merged(assets: &AssetsDir, base: &str) -> Result<MeshBuffer> {
	convert_wavefront_all(&parse_wavefront(assets, base)?)
}

/// Like `load_wavefront_merged`, but individual wavefront Objects
/// are kept separate. (Intended to be lightmapped independently).
pub fn load_wavefront_shards(assets: &AssetsDir, base: &str) -> Result<Vec<MeshBuffer>> {
	convert_wavefront_shards(&parse_wavefront(assets, base)?)
}

/// Find and parse a wavefront file by name (no extension, e.g. "rocket").
fn parse_wavefront(assets: &AssetsDir, base: &str) -> Result<wavefrontobj::ObjSet> {
	let path = assets.find_obj(base)?;
	log_loading(assets, &path);
	match path.extension().unwrap_or_default().to_string_lossy().as_ref() {
		"obj" => wavefrontobj::parse(open(&path)?),
		"gz" => wavefrontobj::parse(GzDecoder::new(open(&path)?)),
		_ => Err(anyhow!("unsupported obj file format: {}", path.to_string_lossy())),
	}
}

pub fn upload_image(ctx: &GraphicsCtx, assets: &AssetsDir, base: &str, sampling: &TextureOpts) -> Result<Texture> {
	Ok(ctx.upload_image_mip(&load_image(assets, base)?, sampling))
}

/// Find and load an image file by base name (no extension, e.g. "lava").
/// Searches `{assets}/textures` for `{base}.png`, `{base.jpg}`, `{base.jpeg}`.
/// Not cached.
//todo resolution
pub fn load_image(assets: &AssetsDir, base: &str) -> Result<DynamicImage> {
	let path = assets.find_texture(base)?;
	log_loading(assets, &path);
	Ok(image::open(&path)?)
}

pub fn log_loading(assets: &AssetsDir, path: &Path) {
	let assets = assets.dir().to_string_lossy();
	let assets = assets.as_ref();
	let path = path.to_string_lossy();
	let msg = path.trim_start_matches(assets);
	LOG.write(format!("loading {}", msg));
}

// Convert a wavefront object to Vertices + Indices that can be uploaded to the GPU.
fn convert_wavefront_all(obj_set: &wavefrontobj::ObjSet) -> Result<MeshBuffer> {
	let shards = convert_wavefront_shards(obj_set)?;
	Ok(MeshBuffer::collect(&shards))
}

/// Convert an Object set into one Meshbuffer per Object.
/// Intended for loading Block models, generating one `Shard` for each surface that needs an independent lightmap.
/// E.g., the 6 faces of a cube would be independent Shards, as the lighting should not
/// be continuous between one face and the next. But the many faces of a icosphere would be a single Shard,
/// because we want the lighting to be continuous over the sphere.
fn convert_wavefront_shards(obj_set: &wavefrontobj::ObjSet) -> Result<Vec<MeshBuffer>> {
	//LOG.write("TODO: tangents for wavefront obj");
	let mut shards = vec![];

	for obj in &obj_set.objects {
		let mut buf = MeshBuffer::new();
		for face in &obj.faces {
			if face.len() != 3 {
				return Err(anyhow!("only triangular faces supported, got face with {} vertices", face.len()));
			}
			for v in face {
				let vertex = VertexLM {
					position: v.position,
					normal: v.normal,
					texcoords: flip_v(v.texture),
					lightcoords: flip_v(v.texture),
					tangent_u: vec3::ZERO,
					tangent_v: vec3::ZERO,
					/*TODO: tangents*/
				};
				buf.push(vertex)
			}
		}
		shards.push(buf)
	}
	// LOG.write(format!("{} SHARDS", shards.len()));
	Ok(shards)
}

pub fn fallback_image() -> RgbImage {
	//image::load_from_memory(include_bytes!("../../../assets/textures/fallback_texture.png"))
	//.expect("decode fallback image")
	RgbImage::from_pixel(2, 2, Rgb([200, 200, 200]))
}

// Flip the orientation of the V texture coordinate.
// Used to convert from Blender's "up" definition or our "up".
fn flip_v(t: vec2) -> vec2 {
	vec2(t.x(), 1.0 - t.y())
}
