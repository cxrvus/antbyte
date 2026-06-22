use antbyte::util::print_error;
use anyhow::Result;

fn main() {
	run().unwrap_or_else(|e| {
		print_error(e);
		std::process::exit(1);
	});
}

pub fn run() -> Result<()> {
	#[cfg(feature = "clap")]
	{
		use antbyte::ui::term;

		if let Some((world, args)) = antbyte::cli::create_world()? {
			term::run(world, args.hide_title);
		}

		Ok(())
	}

	#[cfg(not(feature = "clap"))]
	{
		use antbyte::{
			parser::compiler::LogConfig,
			ui::term,
			world::{World, file_compiler::compile_world},
		};

		use anyhow::Context;

		let args: Vec<String> = std::env::args().collect();

		if args.len() < 2 {
			anyhow::bail!("Usage: {} <PATH>", args[0]);
		}

		let path = std::path::PathBuf::from(&args[1]);

		let properties = compile_world(&path, &LogConfig::default(), &None)?;
		let world = World::new(properties.clone()).context("world error!")?;

		term::run(world, true);

		Ok(())
	}
}
