#![cfg(feature = "clap")]

use std::{
	fs,
	path::{Path, PathBuf},
};

use anyhow::{Context, Ok, Result};
use clap::{self, Parser};

use crate::{
	parser::compiler::LogConfig,
	plugins::{Plugins, render::term_render::TermRenderer},
	world::{World, config::WorldConfig, file_compiler::compile_world},
};

mod args;

use args::Args;

pub fn run() -> Result<()> {
	let args = Args::parse();

	let log_config = LogConfig { all: args.debug };
	let mut properties = compile_world(&args.path, &log_config, &args.sub_args)?;

	if args.json {
		let mut json_path = args.path.clone();
		json_path.set_extension("ant.json");

		// idea: remove properties with default values
		let json = serde_json::to_string(&properties)?;

		fs::write(&json_path, json).with_context(|| {
			format!("failed to write JSON world file to {}", json_path.display())
		})?;
	}

	if args.preview {
		let WorldConfig { width, height, .. } = properties.config;
		let preview_str = "\\/\n".repeat(height) + "|_" + &">>".repeat(width) + "\n\n";
		print!("{preview_str}");
	} else if args.debug {
		// logging happens on compilation
	} else {
		args.set_config(&mut properties.config)
			.context("config-arg error!")?;
		let mut world = World::new(properties.clone()).context("world error!")?;

		let renderer = TermRenderer::new(&world);

		let mut plugins = Plugins {
			renderer: Box::new(renderer),
		};

		if let Some(target) = args.gif {
			export_gif(world, &args.path, target).context("GIF export error!")?;
		} else {
			world.run(&mut plugins).context("world error!")?;
		}
	}

	Ok(())
}

#[rustfmt::skip]
fn export_gif(world: World, source: &Path, target: Option<PathBuf>) -> Result<()> {
	#[cfg(feature = "extras")] { world.gif_export(source, target) }
	#[cfg(not(feature = "extras"))] { _ = (world, source, target); anyhow::bail!("need to enable the `extras` feature-flag in the antbyte crate"); }
}
