use anyhow::Result;

#[inline]
/// need to conventionally make all anyhow context messages end in a '!'
fn print_error(e: anyhow::Error) {
	eprintln!("{}", format!("<!> {e:#}").replace("!: ", ":\n    "));
}

fn main() {
	run().unwrap_or_else(|e| {
		print_error(e);
		std::process::exit(1);
	});
}

pub fn run() -> Result<()> {
	#[cfg(feature = "clap")]
	{
		antbyte::cli::run()
	}

	#[cfg(not(feature = "clap"))]
	{
		use antbyte::{
			parser::compiler::LogConfig,
			ui::{PluginSet, render::term_render::TermRenderer},
			world::{World, file_compiler::compile_world},
		};

		use anyhow::Context;

		let args: Vec<String> = std::env::args().collect();

		if args.len() < 2 {
			anyhow::bail!("Usage: {} <PATH>", args[0]);
		}

		let path = std::path::PathBuf::from(&args[1]);

		let properties = compile_world(&path, &LogConfig::default(), &None)?;
		let mut world = World::new(properties.clone()).context("world error!")?;

		let renderer = TermRenderer::new(&world, true);
		let mut plugins = PluginSet {
			renderer: Box::new(renderer),
			..Default::default()
		};
		world.run(&mut plugins).context("world error!")
	}
}
