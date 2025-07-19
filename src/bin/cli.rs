use std::io::{self, IsTerminal, Read};

use antbyte::{
	ant::{
		archetype::{AntType, Archetype},
		parser::Parser,
		peripherals::{Input, InputType, Output, OutputType, PeripheralSet},
	},
	world::{World, WorldConfig},
};
use anyhow::Result;

fn main() {
	if io::stdin().is_terminal() {
		eprintln!("<!> no pipe input detected. please pipe data into this command.");
		std::process::exit(1);
	}

	println!("<<ANTBYTE>>\n");

	let mut buffer = String::new();

	match io::stdin().read_to_string(&mut buffer) {
		Ok(_) => execute(buffer).unwrap_or_else(|e| eprintln!("<!> {e:?}")),
		Err(e) => {
			eprintln!("error reading from stdin: {e}");
			std::process::exit(1);
		}
	}
}

fn execute(code: String) -> Result<()> {
	println!("{code}");

	let world = create_world(code)?;

	println!("{}\n", world_to_string(&world));

	Ok(())
}

fn create_world(code: String) -> Result<World> {
	use InputType::*;
	use OutputType::*;

	let inputs: Vec<Input> = vec![Input::new(Random, 3)?];
	let inputs = PeripheralSet::inputs(inputs)?;

	let outputs: Vec<Output> = vec![Output::new(Direction, 3)?];
	let outputs = PeripheralSet::outputs(outputs)?;

	let circuit = Parser::parse(code)?;

	let archetype = Archetype::new(AntType::Worker, circuit, inputs, outputs)?;

	let mut config = WorldConfig::default();
	config.archetypes.push(archetype);
	let world = World::new(config);

	Ok(world)
}

fn world_to_string(world: &World) -> String {
	world
		.cells
		.values
		.iter()
		.map(|cell| if *cell == 0 { ".." } else { "##" })
		.collect::<Vec<_>>()
		.chunks(world.cells.width)
		.map(|chunk| chunk.join(""))
		.collect::<Vec<_>>()
		.join("\n")
}
