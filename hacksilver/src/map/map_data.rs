pub use super::internal::*;

/// Map data, can be serialized/deserialized.
///
/// To be able to actually play a Map, we first load the MapData from disk,
/// and then convert it Vertex Arrays, Textures, etc. that live on the GPU.
/// The GPU data cannot be serialized, hence the need to store it separately from `Map`.
#[derive(Clone)]
pub struct MapData {
	pub blocks_by_zone: HashMap<ivec3, Vec<Block>>,

	pub meta: Metadata,
	pub palette: Palette,
}

impl MapData {
	pub fn load(map_dir: &MapDir) -> Result<Self> {
		let block_list = Self::load_blocks(map_dir)?;
		let blocks_by_zone = group_by(block_list.into_iter(), |block| zone_for(block));
		let meta = Metadata::load(map_dir).unwrap_or_default();
		let palette = Self::load_palette_with_default(map_dir);
		Ok(Self { blocks_by_zone, palette, meta })
	}

	fn load_blocks(map_dir: &MapDir) -> Result<Vec<Block>> {
		load_bincode_gz(&map_dir.blocks_file())
	}

	fn load_palette_with_default(map_dir: &MapDir) -> Palette {
		match Self::load_palette(map_dir) {
			Ok(v) => v,
			Err(e) => {
				LOG.write(format!("Error loading palette: {:?}, using defaults", e));
				Palette::default()
			}
		}
	}

	fn load_palette(map_dir: &MapDir) -> Result<Palette> {
		let file = map_dir.palette_file();
		LOG.write(format!("loading {file:?}"));
		Ok(serde_json::from_reader(open(&file)?)?)
	}

	pub fn save(&self, map_dir: &MapDir) -> Result<()> {
		self.save_blocks(map_dir)?;
		self.save_metadata(map_dir)?;
		self.save_palette(map_dir)?;

		Ok(())
	}

	fn save_blocks(&self, map_dir: &MapDir) -> Result<()> {
		let file = map_dir.blocks_file();
		save_bincode_gz(&self.blocks().collect::<Vec<_>>(), &file)
	}

	fn save_metadata(&self, map_dir: &MapDir) -> Result<()> {
		self.meta.save(map_dir)
	}

	fn save_palette(&self, map_dir: &MapDir) -> Result<()> {
		let file = map_dir.palette_file();
		LOG.write(format!("saving {file:?}"));
		Ok(serde_json::to_writer(create(&file)?, &self.palette)?)
	}

	pub fn blocks(&self) -> impl Iterator<Item = Block> + '_ {
		self.blocks_as_ref().cloned()
	}

	pub fn blocks_as_ref(&self) -> impl Iterator<Item = &Block> + '_ {
		self.blocks_by_zone.values().flatten()
	}

	pub fn remove(&mut self, b: &Block) -> Result<()> {
		let zone = zone_for(b);
		let zone = self.blocks_by_zone.get_mut(&zone).ok_or_else(no_such_block)?;
		let i = zone.iter().position(|el| el == b).ok_or_else(no_such_block)?;
		zone.swap_remove(i);
		Ok(())
	}

	pub fn push(&mut self, b: Block) {
		self.blocks_by_zone.entry(zone_for(&b)).or_default().push(b);
	}

	pub fn blocks_ibounds_contain_point(&self, point: ivec3) -> bool {
		let zone = trunc_to_zone(point);
		let empty = vec![];
		for block in self.blocks_by_zone.get(&zone).unwrap_or(&empty) {
			if block.ibounds().contains_excl(point) {
				return true;
			}
		}
		false
	}
}

impl Default for MapData {
	fn default() -> Self {
		Self {
			blocks_by_zone: default(),
			meta: default(),
			palette: default(),
		}
	}
}

fn no_such_block() -> anyhow::Error {
	anyhow!("no such block")
}

//pub fn save_blocks(blocks: Vec<Block>, file: &Path) -> Result<()> {
//	save_bincode_gz(&blocks, file)
//}
//
//pub fn load_blocks(file: &Path) -> Result<Vec<Block>> {
//	load_bincode_gz(file)
//}

pub fn save_bincode_gz<T>(data: &T, file: &Path) -> Result<()>
where
	T: Serialize,
{
	LOG.write(format!("saving {file:?}"));
	Ok(bincode::serialize_into(GzEncoder::new(create(&file)?, flate2::Compression::best()), data)?)
}

pub fn load_bincode_gz<T>(file: &Path) -> Result<T>
where
	T: DeserializeOwned,
{
	LOG.write(format!("loading {file:?}"));
	Ok(bincode::deserialize_from(GzDecoder::new(open(&file)?))?)
}
