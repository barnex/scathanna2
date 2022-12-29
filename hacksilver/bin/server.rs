use anyhow::Result;
use clap::Parser;
use hacksilver::game::*;
use hacksilver::internal::*;
use hacksilver::resources::*;

/// Dedicated server.
#[derive(Parser, Debug)]
struct ServerFlags {
	/// Override the server address. E.g. `192.168.0.1:3456`. Use `server.addr` from settings.toml by default.
	#[arg(long)]
	addr: Option<String>,

	#[arg(long)]
	maplist: Option<Vec<String>>,

	#[arg(long)]
	settings: Option<String>,
}

fn main() {
	env_logger::init();
	let args = ServerFlags::parse();

	debug_warn();

	exit_on_error(main_result(args))
}

fn main_result(args: ServerFlags) -> Result<()> {
	let settings_file = args.settings.as_ref().map(|v| v.as_str()).unwrap_or("server.toml");
	let opts = load_settings(&settings_file)?;
	let opts = override_server_settings(opts, args);
	NetServer::listen_and_serve(opts)
}

fn load_settings(file: &str) -> Result<ServerOpts> {
	let assets = AssetsDir::find()?;
	load_toml(&assets.settings_file(file)?)
}

fn override_server_settings(opts: ServerOpts, flags: ServerFlags) -> ServerOpts {
	let mut settings = opts;
	if let Some(addr) = flags.addr {
		settings.addr = addr;
	}
	if let Some(maplist) = flags.maplist {
		settings.maplist = maplist;
	}
	settings
}
