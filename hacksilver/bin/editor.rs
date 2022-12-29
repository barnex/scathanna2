use anyhow::Result;
use clap::Parser;
use hacksilver::editor::Editor;
use hacksilver::internal::*;
use hacksilver::resources::*;

/// Map editor.
#[derive(Parser)]
struct EditFlags {
	/// Create a new map
	#[arg(long)]
	create: bool,

	/// Path to alternative `settings.toml` file
	#[arg(long, default_value = "settings.toml")]
	settings: String,

	/// Map name (e.g. "my_map").
	map: String,
}

fn main() {
	env_logger::init();
	let args = EditFlags::parse();

	debug_warn();

	exit_on_error(main_result(args))
}

fn main_result(args: EditFlags) -> Result<()> {
	let settings = load_settings(&args.settings)?;
	if args.create {
		Editor::create(&args.map)?;
	}
	Shell::main_loop(settings.graphics, move |ctx| Editor::load(ctx, &args.map))
}

fn load_settings(file: &str) -> Result<Settings> {
	let assets = AssetsDir::find()?;
	load_toml(&assets.settings_file(file)?)
}
