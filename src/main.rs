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
		antbyte::cli::command_parser::run()
	}

	#[cfg(not(feature = "clap"))]
	{
		use antbyte::{
			ant::{compiler::LogConfig, world::World},
			file_compiler::compile_world_file,
		};

		use anyhow::Context;

		let args: Vec<String> = std::env::args().collect();

		if args.len() < 2 {
			anyhow::bail!("Usage: {} <PATH>", args[0]);
		}

		let path = std::path::PathBuf::from(&args[1]);

		let properties = compile_world_file(&path, &LogConfig::default(), &None)?;
		let mut world = World::new(properties).context("world error!")?;

		world.run().context("world error!")
	}
}
