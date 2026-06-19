use std::collections::BTreeMap;

use crate::{
	util::vec2::Pos,
	world::{
		World,
		config::{RenderMask, WorldConfig},
	},
};

pub fn run(world: World) {
	let mut world = world;

	print!("\n\n");

	while let Some(frame) = world.next_frame_auto() {
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

		// ## Metadata
		println!("\n{}\n", world.metadata_str());
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
