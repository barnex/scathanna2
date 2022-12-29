use super::internal::*;
use std::fs;

/// BufReader for reading file with more descriptive message on error.
pub fn open(file: &Path) -> Result<impl Read> {
	log::info!("loading {}", file.to_string_lossy());
	Ok(BufReader::new(File::open(file).map_err(|err| anyhow!("open {:?}: {}", file, err))?))
}

/// BufWriter for writing file with more descriptive message on error.
#[allow(dead_code)]
pub fn create(file: &Path) -> Result<impl Write> {
	log::info!("writing {}", file.to_string_lossy());
	Ok(BufWriter::new(File::create(file).map_err(|err| anyhow!("create {:?}: {}", file, err))?))
}

/// Read file names (no full path) in a directory.
pub fn read_dir_names(path: &Path) -> Result<impl Iterator<Item = PathBuf>> {
	Ok(fs::read_dir(path)
		.map_err(|e| anyhow!("read '{path:?}': {e}"))? //
		.filter_map(|entry| entry.ok())
		.map(|entry| PathBuf::from(entry.file_name())))
}

pub fn mkdir(path: impl AsRef<Path>) -> Result<()> {
	let path = path.as_ref();
	fs::create_dir(path).map_err(|e| anyhow!("create directory '{path:?}': {e}"))
}
