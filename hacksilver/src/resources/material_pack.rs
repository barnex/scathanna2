pub use super::internal::*;
use std::fs;

/// A loader and texture cache PBR materials.
pub struct MaterialPack {
	ctx: Arc<GraphicsCtx>,
	assets: AssetsDir,
	available_names: Vec<String>,
	host_cache: Mutex<HashMap<String, Arc<HostMaterial>>>,
	cache: Mutex<HashMap<String, GMaterial>>,
}

impl MaterialPack {
	pub fn new(ctx: &Arc<GraphicsCtx>, assets: AssetsDir) -> Result<Self> {
		let ctx = ctx.clone();

		let mat_dir = assets.materials_dir(ctx.opts.texture_resolution);
		let available_names = match ctx.opts.textures_enabled() {
			true => Self::find_material_names(&mat_dir)?,
			false => vec![],
		};

		Ok(Self {
			ctx,
			available_names,
			assets,
			cache: default(),
			host_cache: default(),
		})
	}

	fn find_material_names(mat_dir: &Path) -> Result<Vec<String>> {
		let available_names = fs::read_dir(&mat_dir)
			.map_err(|e| anyhow!("read '{:?}': {}", mat_dir, e))?
			.filter_map(|entry| entry.ok())
			.filter(|entry| entry.file_type().map(|typ| typ.is_dir()).unwrap_or(false))
			.filter_map(|entry| entry.file_name().to_str().map(str::to_owned))
			.collect::<Vec<_>>()
			.with(|names| names.sort_by_key(|name| name.to_ascii_lowercase()));
		if available_names.is_empty() {
			return Err(anyhow!("no materials found in `{mat_dir:?}`"));
		}
		Ok(available_names)
	}

	pub fn get(&self, name: &str) -> GMaterial {
		// cache hit
		if let Some(mat) = self.cache.lock().unwrap().get(name) {
			return mat.clone();
		}

		if self.cache.lock().unwrap().len() > 16 {
			self.clear_cache()
		}

		// miss:
		let mat = GMaterial::upload(&self.ctx, &self.get_host(name));
		self.cache.lock().unwrap().insert(name.to_owned(), mat.clone());
		mat
	}

	pub fn clear_cache(&self) {
		self.cache.lock().unwrap().clear()
	}

	pub fn get_host(&self, name: &str) -> Arc<HostMaterial> {
		// cache hit
		if let Some(mat) = self.host_cache.lock().unwrap().get(name) {
			return mat.clone();
		}

		let mat = match HostMaterial::load(&self.ctx.opts, &self.materials_dir(), name) {
			Ok(mat) => mat,
			Err(e) => {
				LOG.write(format!("error loading material `{}`: {}", name, e));
				HostMaterial::fallback()
			}
		};
		let mat = Arc::new(mat);

		self.host_cache.lock().unwrap().insert(name.to_owned(), mat.clone());
		mat
	}

	fn materials_dir(&self) -> PathBuf {
		self.assets.materials_dir(self.ctx.opts.texture_resolution)
	}

	pub fn fallback(&self) -> GMaterial {
		GMaterial::fallback(&self.ctx)
	}

	pub fn host_fallback(&self) -> HostMaterial {
		HostMaterial::fallback()
	}

	pub fn next_after(&self, name: &str, delta: i32) -> &str {
		let i = self.available_names.iter().position(|el| el == name).unwrap_or_default();
		let mut next = i as i32 + delta;
		if next < 0 {
			next = self.available_names.len() as i32 - 1;
		}
		if next >= self.available_names.len() as i32 {
			next = 0;
		}
		self.available_names.get(next as usize).expect("materials available")
	}
}
