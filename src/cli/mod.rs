#![cfg(feature = "clap")]

use std::{
	fs,
	path::{Path, PathBuf},
};

use anyhow::{Context, Ok, Result};
use clap::{self, Parser};

use crate::{
	parser::compiler::LogConfig,
	ui::term,
	world::{World, config::WorldConfig, file_compiler::compile_world},
};

mod args;

use args::Args;

pub fn run() -> Result<()> {
	let args = Args::parse();

	let log_config = LogConfig { all: args.debug };
	let mut properties = compile_world(&args.path, &log_config, &args.sub_args)?;

	if args.json {
		// idea: remove properties with default values
		let json = serde_json::to_string_pretty(&properties)?;
		println!("{json}");
	}

	if args.preview {
		let WorldConfig { width, height, .. } = properties.config;
		let preview_str =
			"\\/\n".repeat(height as usize) + "|_" + &">>".repeat(width as usize) + "\n\n";
		print!("{preview_str}");
	} else if args.debug || args.json {
		// logging happens on compilation
	} else {
		args.set_config(&mut properties.config)
			.context("config-arg error!")?;
		let world = World::new(properties.clone()).context("world error!")?;

		if let Some(target) = args.gif {
			export_gif(world, &args.path, target).context("GIF export error!")?;
		} else if args.raw {
			term::raw::run(world);
		} else {
			term::run(world, args.hide_title);
		}
	}

	Ok(())
}

#[rustfmt::skip]
fn export_gif(world: World, source: &Path, target: Option<PathBuf>) -> Result<()> {
	#[cfg(feature = "extras")] { crate::gif_export::gif_export(&world, source, target) }
	#[cfg(not(feature = "extras"))] { _ = (world, source, target); anyhow::bail!("need to enable the `extras` feature-flag in the antbyte crate"); }
}
