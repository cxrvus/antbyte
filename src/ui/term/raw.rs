use std::collections::BTreeMap;

use crate::{
	ui::chars_to_input,
	util::vec2::Pos,
	world::{
		World,
		config::{RenderMask, WorldConfig},
		frame::FrameInput,
	},
};

pub fn run(world: World) {
	let mut world = world;

	print!("\n\n");

	let mut input_str = String::new();
	let mut input = FrameInput::default();

	while let Some(frame) = world.next_frame(&input) {
		// ## FG
		if let RenderMask::None = world.config().fg {
			println!("--");
		} else {
			print_grid(world.config(), &frame.fg);
		}

		println!();

		// ## BG
		if let RenderMask::None = world.config().bg {
			println!("--");
		} else {
			print_grid(world.config(), &frame.bg);
		}

		println!();

		// ## Metadata
		let ms = frame.ms.map(|ms| ms.to_string()).unwrap_or("--".into());
		println!("t: {ms}");

		let metadata = world.metadata_str();
		println!("{metadata}");
		println!();

		// ## External Input
		input_str.clear();
		std::io::stdin().read_line(&mut input_str).unwrap();
		input.ext_in = chars_to_input(&world.config().keys, &input_str);
	}
}

fn print_grid(config: &WorldConfig, grid: &BTreeMap<Pos, u8>) {
	for y in 0..config.height {
		for x in 0..config.width {
			let value = grid.get(&Pos { x, y });

			if let Some(value) = value {
				print!("{value:02x}",);
			} else {
				print!("..")
			}
		}

		println!();
	}
}
