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
	let mut properties = compile_world_file(&args.path, &log_config)?;

	if args.preview {
		let WorldConfig { width, height, .. } = properties.config;
		let preview_str = "\\/\n".repeat(height) + "|_" + &">>".repeat(width) + "\n\n";
		print!("{preview_str}");
		Ok(())
	} else if !args.debug {
		set_config(&mut properties.config, &args);

		let mut world = World::new(properties).context("world error!")?;

		world.run().context("world error!")?;

		Ok(())
	} else {
		Ok(())
	}
}

#[rustfmt::skip]
fn set_config(config: &mut WorldConfig, args: &Args) {
	if args.stepped { config.fps = None; }
	if args.instant { config.tpf = None; }
	if args.looping { config.looping = true; }
	if args.ticks.is_some() { config.ticks = args.ticks; }
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
