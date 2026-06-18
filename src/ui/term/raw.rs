use crate::{
	util::vec2::Position,
	world::{World, config::RenderMask},
};

pub fn run(world: World) {
	let mut world = world;
	while let Some(frame) = world.next_frame_auto() {
		// ## FG
		if let RenderMask::None = world.config().fg {
			println!("--");
		} else {
			for y in 0..world.config().height {
				for x in 0..world.config().width {
					let fg_value = frame
						.fg
						.get(&Position { x, y })
						.copied()
						.unwrap_or_default();

					print!("{:02x}", fg_value);
				}
				println!();
			}
		}

		print!("\n\n");

		// ## BG
		for y in 0..world.config().height {
			for x in 0..world.config().width {
				let bg_value = frame
					.bg
					.get(&Position { x, y })
					.copied()
					.unwrap_or_default();

				print!("{:02x}", bg_value);
			}
			println!();
		}

		// ## Metadata
		println!("\n{}\n", world.metadata_str());
		print!("\n\n\n");
	}
}
