use super::{World, WorldConfig};
use anyhow::Result;
use std::io::{self, Write};

impl World {
	pub fn run(&mut self) -> Result<()> {
		let mut fps = self.properties.config.fps;

		loop {
			println!("\n<<ANTBYTE>>\n===========\n\n");
			println!("{:0>10}", self.frame());
			println!("{}\n\n", self.full_render());

			if fps == 0 {
				io::stderr().flush().unwrap();
				let mut input = String::new();

				io::stdin().read_line(&mut input).unwrap();
				if input.trim() == "a" {
					fps = WorldConfig::default().fps;
				}
			}

			let world_active = self.tick();

			if !world_active {
				return Ok(());
			}
		}
	}

	fn full_render(&self) -> String {
		let cells = &self.cells;
		let mut string = String::new();

		for (i, cell) in cells.values.iter().enumerate() {
			if i % cells.width == 0 {
				string.push('\n');
			}

			let pos = cells.get_pos(i).unwrap();
			let ant = self.ants().iter().find(|ant| ant.pos == pos);

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
