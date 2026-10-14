#![cfg(feature = "clap")]

use std::path::Path;

use anyhow::{Context, Result, bail};
use clap::{self, Parser};

use crate::{
	parser::compiler::LogConfig,
	ui::{RenderMode, raw},
	world::{World, config::WorldConfig, file_compiler::compile_world},
};

mod args;

use args::Args;

pub fn run_world() -> Result<()> {
	let args = Args::parse();
	let path = args.path()?;
	let log_config = LogConfig { all: args.debug };
	let mut properties = compile_world(&path, &log_config, &args.sub_args)?;

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

		let render_mode = match args.render {
			Some(render_mode) => match render_mode.as_str() {
				"" => bail!("must provide a render mode"),
				render_mode => RenderMode::try_from(render_mode)?,
			},
			None => Default::default(),
		};

		// TODO: simulation error
		match render_mode {
			RenderMode::Gui => {
				#[cfg(not(feature = "gui"))]
				{
					anyhow::bail!("need to enable the gui feature flag to render in GUI")
				}
				#[cfg(feature = "gui")]
				{
					crate::ui::gui::run(world, args.fullscreen)
						.map_err(|err| anyhow::anyhow!(err.to_string()))
						.context("in GUI!")?
				}
			}

			RenderMode::Raw => raw::run(world, false),
			RenderMode::None => raw::run(world, true),
			RenderMode::Gif => export_gif(world, &path).context("GIF export error!")?,
			RenderMode::Term => todo!(),
		};
	}
	Ok(())
}

#[rustfmt::skip]
fn export_gif(world: World, source: &Path) -> Result<()> {
	#[cfg(feature = "gif")] { crate::gif_export::gif_export(&world, source) }
	#[cfg(not(feature = "gif"))] { _ = (world, source); anyhow::bail!("need to enable the `gif` feature-flag in the antbyte crate"); }
}
