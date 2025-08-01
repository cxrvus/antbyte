use std::env;
use std::fs;
use std::io::{self, Write};

use antbyte::circuit::Circuit;
use antbyte::parser::parse;
use antbyte::{
	ant::{
		archetype::{AntType, Archetype},
		peripherals::{Input, InputType, Output, OutputType, PeripheralSet},
	},
	world::{World, WorldConfig},
};
use anyhow::{Result, anyhow};

fn main() {
	setup().unwrap_or_else(|e| {
		eprintln!("<!> {e:?}");
		std::process::exit(1);
	});
}

/// use this for debugging
// TODO
const TEST_PATH: Option<&'static str> = Some("ant_code/test.ant");
// const TEST_PATH: Option<&'static str> = None;

/// use this for debugging
const AUTO_LOOP: bool = false;

fn setup() -> Result<()> {
	let args: Vec<String> = env::args().collect();

	if TEST_PATH.is_none() && args.len() != 2 {
		return Err(anyhow!("Usage: {} <ant_file>", args[0]));
	}

	let path = match TEST_PATH {
		Some(path) => path,
		None => &args[1],
	};

	if !path.ends_with(".ant") {
		return Err(anyhow!("ant files need to have the .ant extension"));
	}

	let code =
		fs::read_to_string(path).map_err(|e| anyhow!("Error reading file {}: {}", args[1], e))?;

	println!("<<ANTBYTE>>\n");

	update(code)
}

fn update(code: String) -> Result<()> {
	println!("{code}");

	// TODO: temporary
	parse(code)?;
	return Ok(());

	let mut world = create_world(code)?;
	let mut auto_loop = AUTO_LOOP;

	loop {
		println!("{:0>10}", world.frame());
		println!("{}\n\n", world_to_string(&world));

		if !auto_loop {
			io::stderr().flush().unwrap();
			let mut input = String::new();

			io::stdin().read_line(&mut input).unwrap();
			if input.trim() == "a" {
				auto_loop = true;
			}
		}

		world.tick();
	}
}

fn create_world(code: String) -> Result<World> {
	use InputType::*;
	use OutputType::*;

	let inputs: Vec<Input> = vec![Input::new(Random, 4)?];
	let inputs = PeripheralSet::inputs(inputs)?;

	let outputs: Vec<Output> = vec![Output::new(Direction, 3)?, Output::new(Spawn, 1)?];
	let outputs = PeripheralSet::outputs(outputs)?;

	let circuit = Circuit::try_from(code)?;

	let archetype = Archetype::new(AntType::Worker, circuit, inputs, outputs)?;

	let mut config = WorldConfig::default();
	config.archetypes.push(archetype);
	let world = World::new(config);

	Ok(world)
}

fn world_to_string(world: &World) -> String {
	let mut string = String::new();

	for (i, cell) in world.cells.values.iter().enumerate() {
		if i % world.cells.width == 0 {
			string.push('\n');
		}

		let pos = world.cells.get_pos(i).unwrap();
		let ant = world
			.ants
			.iter()
			.filter(|ant| ant.alive)
			.find(|ant| ant.pos == pos);

		let cell_char = match cell {
			0 => '.',
			_ => '#',
		};

		let ant_char = match ant {
			None => cell_char,
			Some(ant) => ant.get_dir_vec().as_char(),
		};

		string.push(ant_char);
		string.push(cell_char);
	}

	string
}
