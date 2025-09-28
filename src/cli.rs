#![cfg(feature = "clap")]

use clap::Parser;
use std::path::PathBuf;

use crate::ant::{
	compiler::{LogConfig, compile_world_file},
	world::{World, WorldConfig},
};

use anyhow::{Context, Ok, Result};

#[derive(Parser, Debug, Default)]
#[command(version, about, long_about = None)]
struct Args {
	/// Path to the .ant file to execute
	path: PathBuf,

	/// Step through the simulation, waiting for input after each frame (FPS = 0)
	#[arg(short, long)]
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
	let args = Args::parse();

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
			set_config(&mut world, &args);
			world.run().context("world error!")?;
		}
	}

	Ok(())
}

#[rustfmt::skip]
fn set_config(world: &mut World, args: &Args) {
	let config = world.config_mut();
	if args.stepped { config.fps = None; }
	if args.instant { config.tpf = None; }
	if args.looping { config.looping = true; }
	if args.ticks.is_some() { config.ticks = args.ticks; }
}

#[rustfmt::skip]
fn export_gif(world: World, opt_path: Option<PathBuf>) -> Result<()> {
	#[cfg(feature = "gif")] { world.gif_export(opt_path) }
	#[cfg(not(feature = "gif"))] { _ = (world, opt_path); anyhow::bail!("need to enable the `gif` feature-flag in the antbyte crate"); }
}

#[inline]
pub fn clear_screen() {
	print!("\x1B[2J\x1B[1;1H");
}

pub fn print_title() {
	let title = r#"
░░      ░░░   ░░░  ░░        ░░       ░░░  ░░░░  ░░        ░░        ░
▒  ▒▒▒▒  ▒▒    ▒▒  ▒▒▒▒▒  ▒▒▒▒▒  ▒▒▒▒  ▒▒▒  ▒▒  ▒▒▒▒▒▒  ▒▒▒▒▒  ▒▒▒▒▒▒▒
▓  ▓▓▓▓  ▓▓  ▓  ▓  ▓▓▓▓▓  ▓▓▓▓▓       ▓▓▓▓▓    ▓▓▓▓▓▓▓  ▓▓▓▓▓      ▓▓▓
█        ██  ██    █████  █████  ████  █████  ████████  █████  ███████
█  ████  ██  ███   █████  █████       ██████  ████████  █████        █
                                                                                                                                                      
	"#;

	println!("{title}");
}

pub fn print_title_short() {
	println!("<<ANTBYTE>>");
}
