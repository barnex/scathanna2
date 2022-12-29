use anyhow::Result;
use clap::Parser;
use hacksilver::game::*;
use hacksilver::internal::*;
use hacksilver::resources::*;

/// Play the game by connecting to a server.
#[derive(Parser)]
struct PlayFlags {
	/// Force connect to this server, instead of using settings.toml.
	#[arg(long)]
	server: Option<String>,

	/// Path to alternative `settings.toml` file
	#[arg(long, default_value = "settings.toml")]
	settings: String,

	/// Override player name.
	#[arg(short, long)]
	name: Option<String>,

	/// Override team (red|green|blue)
	#[arg(short, long)]
	team: Option<String>,

	/// Force disable sound (overrides settings.toml).
	#[arg(long)]
	no_sound: bool,
}

fn main() {
	env_logger::init();
	let args = PlayFlags::parse();

	debug_warn();

	exit_on_error(main_result(args));
}

fn main_result(args: PlayFlags) -> Result<()> {
	// Load settings.toml.
	// If there's an error, we still want to show an error screen,
	// but we don't have the GraphicsOptions to do so (e.g. for window size), so we must use defaults.
	let settings = match load_settings(&args.settings) {
		Ok(settings) => settings,
		Err(e) => return Shell::main_loop(GraphicsOpts::default(), move |_| -> Result<NopApp> { Err(anyhow!("{}: {e:#}", args.settings)) }),
	};

	let settings = settings.with(|s| override_play_settings(s, args));
	Shell::main_loop(settings.graphics.clone(), move |ctx| Client::new(ctx, settings))
}

fn load_settings(file: &str) -> Result<Settings> {
	let assets = AssetsDir::find()?;
	load_toml(&assets.settings_file(file)?)
}

fn override_play_settings(settings: &mut Settings, flags: PlayFlags) {
	if flags.no_sound {
		settings.sound.enabled = false;
	}
	if let Some(server) = flags.server {
		settings.network.servers = vec![server];
	}
	if let Some(name) = flags.name {
		settings.player.name = name;
	}
	if let Some(team) = flags.team {
		settings.player.team = team;
	}
}

// An App that does nothing. Used to if there was an error loading settings.toml.
// This is a bit of a hack, but hard to do otherwise since we need settings.toml to bootstrap a graphical context.
struct NopApp {}

impl App for NopApp {
	fn handle_tick(&mut self, _inputs: &Inputs) -> StateChange {
		StateChange::None
	}

	fn handle_draw(&self, viewport: uvec2) -> SceneGraph {
		SceneGraph::new(viewport)
	}
}
