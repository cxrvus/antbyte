use std::path::PathBuf;

use antbyte::ant::{
	compiler::{LogConfig, compile_world_file},
	world::{World, WorldConfig},
};

use anyhow::{Context, Ok, Result};
use clap::Parser;

fn main() {
	setup().unwrap_or_else(|e| {
		// need to conventionally provide all anyhow context messages ending in a '!'
		eprintln!("{}", format!("<!> {e:#}").replace("!: ", ":\n    "));
		std::process::exit(1);
	});
}

#[derive(Parser, Debug, Default)]
#[command(version, about, long_about = None)]
struct Args {
	/// Path to the .ant file to execute
	path: PathBuf,

	/// Step through the simulation, waiting for input after each frame (same as setting FPS = 0)
	#[arg(short, long)]
	stepped: bool,

	// todo: make this a world property
	/// Log debug info instead of running the simulation
	#[arg(short, long)]
	log: bool,

	/// Show a preview of the dimensions of the antlet
	#[arg(short, long)]
	preview: bool,
}

fn setup() -> Result<()> {
	let args = Args::parse();

	let log_config = LogConfig { all: args.log };
	let mut properties = compile_world_file(&args.path, &log_config)?;

	if args.preview {
		let WorldConfig { width, height, .. } = properties.config;
		let preview_str = "\\/\n".repeat(height) + "|_" + &">>".repeat(width) + "\n\n";
		print!("{preview_str}");
		Ok(())
	} else if !args.log {
		if args.stepped {
			properties.config.fps = None;
		}

		let mut world = World::new(properties).context("world error!")?;

		world.run();

		Ok(())
	} else {
		Ok(())
	}
}
