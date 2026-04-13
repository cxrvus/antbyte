use crate::{
	parser::{Parser, token::Token},
	world::config::WorldConfig,
};

use clap::{self, Parser as ClapParser};
use std::path::PathBuf;

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

	/// Hide Title Banner
	#[arg(short = 'T', long)]
	hide_title: bool,

	/// Hide Ants
	#[arg(short = 'A', long)]
	hide_ants: bool,

	/// output data in machine-readable format
	#[arg(short, long)]
	pub raw: bool,

	/// pass args to sub-process, e.g. a nodejs file
	#[arg(short = 'a', long = "args")]
	pub sub_args: Option<String>,

	/// create a JSON world file upon compilation
	#[arg(short, long)]
	pub json: bool,

	// idea: turn these into sub-commands, since the config args are ignored anyway
	/// Export as GIF
	#[arg(long)]
	pub gif: Option<Option<PathBuf>>,

	/// Log debug info instead of running the simulation
	#[arg(short, long)]
	pub debug: bool,

	/// Show a preview of the dimensions of the antlet
	#[arg(short, long)]
	pub preview: bool,
}

impl Args {
	#[rustfmt::skip]
	pub fn set_config(&self, config: &mut WorldConfig) -> anyhow::Result<()> {
		if let Some(cfg) = &self.cfg {
			let cfg = if cfg.trim().ends_with(';') { cfg } else { &format!("{cfg};") };

			let mut parser = Parser::new(cfg)?;

			while !parser.assume_next(Token::EndOfFile) {
				let (key, value) = parser.parse_setting()?;
				config.set_setting(key, value)?;
			}
		}

		if self.hide_title { config.hide_title = true; }
		if self.hide_ants { config.hide_ants = true; }
		if self.stepped { config.fps = None; }
		if self.instant { config.speed = None; }
		if self.looping { config.looping = true; }
		if self.ticks.is_some() { config.ticks = self.ticks; }

		if self.gif.is_some() && (config.speed.is_none() | config.fps.is_none()) {
			anyhow::bail!("need a speed and an FPS of at least 1 to export as GIF");
		}

		Ok(())
	}
}
