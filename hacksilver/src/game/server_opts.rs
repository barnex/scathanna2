use super::internal::*;
use clap::Parser;

/// Command-line options for game server.
#[derive(Parser, Debug, Serialize, Deserialize)]
pub struct ServerOpts {
	#[arg(short, long, default_value = "127.0.0.1:3344")]
	pub addr: String,

	#[arg(short, long)]
	pub maplist: Vec<String>,

	pub frag_limit: u32,

	pub time_limit: u32,
}

impl Default for ServerOpts {
	fn default() -> Self {
		Self {
			addr: "127.0.0.1:3344".into(),
			maplist: vec![],
			frag_limit: 20,
			time_limit: 460,
		}
	}
}
