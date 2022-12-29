use super::internal::*;
use std::fs;

/// Powers the editor's recording functionality:
/// added blocks can be recorded into a template,
/// then copied into the scene multiple times.
///
/// E.g. record building a nice castle tower,
/// then instantiate many more such towers.
#[derive(Default)]
pub struct Recording {
	/// Recording in progress
	current_recording: Option<(MapDir, String, Set<Block>)>,
	// Removed blocks are recorded here,
	// so they can be pasted back later.
	//cutted: Set<Block>,
}

/// Recordings directory (inside map directory).
//const REC: &'static str = "rec";

impl Recording {
	/// To be called when a block was added.
	/// If a recording is active, the block will be added to it.
	pub fn record_add(&mut self, b: Block) {
		if let Some((_, _, rec)) = self.current_recording.as_mut() {
			rec.insert(b);
		}
		//self.cutted.remove(&b);
	}

	/// To be called when a block was removed.
	/// If a recording is active, the block will be removed from it.
	pub fn record_remove(&mut self, b: &Block) {
		if let Some((_, _, rec)) = self.current_recording.as_mut() {
			rec.remove(b);
		}
		//self.cutted.insert(b.clone());
	}

	/// Start recording add/remove under the given record name.
	pub fn start_recording(&mut self, map_dir: MapDir, name: &str) {
		LOG.write(format!("recording `{name}`"));
		self.current_recording = Some((map_dir, name.to_owned(), default()));
	}

	fn file_for(map_dir: &MapDir, name: &str) -> PathBuf {
		map_dir.recordings().join(name).with_extension("bincode.gz")
	}

	/// Stop recording, return the name of the completed recording if any was in progress.
	pub fn stop_recording(&mut self) -> Result<String> {
		if let Some((dir, name, blocks)) = self.current_recording.take() {
			if blocks.len() != 0 {
				let file = Self::file_for(&dir, &name);
				let _ = fs::create_dir(file.parent().unwrap());
				save_bincode_gz(&blocks.iter().copied().collect::<Vec<_>>(), &file)?;
			}
			Ok(name)
		} else {
			Err(anyhow!("no such recording"))
		}
	}

	/// Get the blocks corresponding to a record name.
	/// Intended to be used as a new cursor template.
	pub fn get(&self, map_dir: &MapDir, name: &str) -> Result<impl Iterator<Item = Block> + '_> {
		let file = Self::file_for(map_dir, name);
		let blocks: Vec<Block> = load_bincode_gz(&file)?;
		Ok(blocks.into_iter())
	}

	/// List all record names.
	pub fn names(&self, map_dir: &MapDir) -> Result<Vec<String>> {
		Ok(fs::read_dir(map_dir.recordings())?
			.map(|r| r.map(|d| d.file_name().to_string_lossy().to_string()))
			.collect::<std::result::Result<Vec<String>, std::io::Error>>()?)
	}

	// Start with a fresh cutting buffer (called by editor>cut).
	// TODO: obsolete
	//pub fn start_cut(&mut self) {
	//	self.cutted.clear()
	//}

	// Return the blocks in the cutting buffer (called by editor>paste).
	//pub fn paste(&mut self) -> impl Iterator<Item = Block> + '_ {
	//	self.cutted.drain()
	//}

	//pub fn record_clipboard(&mut self, name: &str) {
	//	self.recordings.insert(name.into(), take(&mut self.cutted));
	//}
}
