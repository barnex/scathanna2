use super::internal::*;
use once_cell::sync::Lazy;

#[derive(Default)]
pub struct LogBuffer {
	lines: Mutex<Vec<String>>,
}

/// Global LogBuffer.
/// Text logged here appears in stdout and the game Shell.
pub static LOG: Lazy<LogBuffer> = Lazy::new(LogBuffer::new);

impl LogBuffer {
	const MAX_LINES: usize = 48;

	pub fn new() -> Self {
		Self::default()
	}

	pub fn write_prefixed<T: AsRef<str>>(&self, prefix: &str, line: T) {
		let line = line.as_ref();
		self.write(format!("{prefix}: {line}"))
	}

	pub fn write<T: AsRef<str>>(&self, line: T) {
		let line = line.as_ref();
		println!("{}", &line);

		let mut lines = self.lines.lock().expect("poisoned");

		// corner case: empty line
		if line == ""{
			lines.push(line.into());
		}

		for line in line.lines() {
			lines.push(line.into());
			if lines.len() > Self::MAX_LINES {
				lines.remove(0);
			}
		}
	}

	pub fn replace_last_line<T: Into<String>>(&self, line: T) {
		let line = line.into();
		println!("\x1B[1F\x1B[0J{line}");
		let mut lines = self.lines.lock().expect("poisoned");

		if let Some(last) = lines.last_mut() {
			*last = line
		} else {
			self.write(line)
		}
	}

	pub fn to_string(&self) -> String {
		let lines = self.lines.lock().expect("poisoned");
		lines.join("\n")
	}
}
