#![cfg(feature = "clap")]

use crate::ant::world::parser::token::Token;
use clap::{self, Parser as ClapParser};
use std::path::PathBuf;

use crate::ant::{
	compiler::{LogConfig, compile_world_file},
	world::{World, WorldConfig, parser::Parser},
};

use anyhow::{Context, Ok, Result};

#[derive(ClapParser, Debug, Default, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
	/// Path to the .ant file to execute
	pub path: PathBuf,

	/// Step through the simulation, waiting for input after each frame (FPS = 0)
	#[arg(short = 'S', long)]
	stepped: bool,

	/// Looping simulation
	#[arg(short, long)]
	looping: bool,

	/// Instant simulation
	#[arg(short, long)]
	instant: bool,

	/// Set tick limit
	#[arg(short, long)]
	ticks: Option<u32>,

	/// Configure settings
	#[arg(short, long)]
	cfg: Option<String>,

	/// watch-mode, combine with -d / --debug for dry-runs
	#[arg(short, long)]
	pub watch: bool,

	// todo: turn these into sub-commands, since the config args are ignored anyway
	/// Export as GIF
	#[arg(long)]
	gif: Option<Option<PathBuf>>,

	/// Log debug info instead of running the simulation
	#[arg(short, long)]
	debug: bool,

	/// Show a preview of the dimensions of the antlet
	#[arg(short, long)]
	preview: bool,
}

pub fn run() -> Result<()> {
	let mut args = Args::parse();

	if args.watch {
		#[cfg(feature = "extras")]
		{
			crate::cli::watch::watch_file(&mut args).context("watch-mode error!")?;
		}
		#[cfg(not(feature = "extras"))]
		{
			anyhow::bail!("watch-mode requires the `extras` feature flag to be enabled");
		}
	} else {
		run_once(args)?;
	}

	Ok(())
}

pub fn run_once(args: Args) -> Result<()> {
	let log_config = LogConfig { all: args.debug };
	let properties = compile_world_file(&args.path, &log_config)?;

	if args.preview {
		let WorldConfig { width, height, .. } = properties.config;
		let preview_str = "\\/\n".repeat(height) + "|_" + &">>".repeat(width) + "\n\n";
		print!("{preview_str}");
	} else if args.debug {
		// logging happens on compilation
	} else {
		let mut world = World::new(properties).context("world error!")?;

		if let Some(opt_path) = args.gif {
			export_gif(world, opt_path).context("GIF export error!")?;
		} else {
			set_config(&mut world, &args).context("config-arg error!")?;
			world.run().context("world error!")?;
		}
	}

	Ok(())
}

#[rustfmt::skip]
fn set_config(world: &mut World, args: &Args) -> Result<()> {
	let config = world.config_mut();
	if args.stepped { config.fps = None; }
	if args.instant { config.speed = None; }
	if args.looping { config.looping = true; }
	if args.ticks.is_some() { config.ticks = args.ticks; }

	if let Some(cfg) = &args.cfg {
		let cfg = if cfg.trim().ends_with(';') { cfg } else { &format!("{cfg};") };

		let mut parser = Parser::new(cfg)?;

		while !parser.assume_next(Token::EndOfFile) {
			let (key, value) = parser.parse_setting()?;
			world.config_mut().set_setting(key, value)?;
		}
	}

	Ok(())
}

#[rustfmt::skip]
fn export_gif(world: World, opt_path: Option<PathBuf>) -> Result<()> {
	#[cfg(feature = "extras")] { world.gif_export(opt_path) }
	#[cfg(not(feature = "extras"))] { _ = (world, opt_path); anyhow::bail!("need to enable the `extras` feature-flag in the antbyte crate"); }
}
