use super::{World, WorldProperties};
use anyhow::{Context, Result};
use std::io::{self, Write};

pub fn run(properties: WorldProperties, auto_step: bool) -> Result<()> {
	let mut auto_step = auto_step;
	let mut world = World::new(properties).context("world error!")?;

	loop {
		println!("\n<<ANTBYTE>>\n===========\n\n");
		println!("{:0>10}", world.age());
		println!("{}\n\n", world_to_string(&world));

		if !auto_step {
			io::stderr().flush().unwrap();
			let mut input = String::new();

			io::stdin().read_line(&mut input).unwrap();
			if input.trim() == "a" {
				auto_step = true;
			}
		}

		let world_active = world.tick();

		if !world_active {
			return Ok(());
		}
	}
}

fn color_codes(value: u8) -> (u8, u8) {
	let color = value & 0b0111;
	let intensity = (value & 0b1000) != 0;
	let bg_color = if intensity { 100 + color } else { 40 + color };
	let flipped_color = color ^ 0b0111;

	let fg_color = if intensity {
		90 + flipped_color
	} else {
		30 + flipped_color
	};

	(bg_color, fg_color)
}

fn color_cell(value: u8, content: &str) -> String {
	let (bg, fg) = color_codes(value);
	format!("\x1b[{fg};{bg}m{content}\x1b[0m")
}

fn world_to_string(world: &World) -> String {
	let cells = &world.cells;
	let mut string = String::new();

	for (i, cell) in cells.values.iter().enumerate() {
		if i % cells.width == 0 {
			string.push('\n');
		}

		let pos = cells.get_pos(i).unwrap();
		let ant = world.ants().iter().find(|ant| ant.pos == pos);

		match ant {
			None => {
				string.push_str(&color_cell(*cell, "  "));
			}
			Some(ant) => {
				let (char1, char2) = ant.dir_vec().principal_chars();
				let ant_chars = format!("{char1}{char2}");
				string.push_str(&color_cell(*cell, &ant_chars));
			}
		}
	}

	string
}
