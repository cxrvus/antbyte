use crate::{util::vec2::Position, world::World};

pub fn run(world: World) {
	let mut world = world;
	while let Some(frame) = world.next_frame_auto() {
		for y in 0..world.config().height {
			for x in 0..world.config().width {
				let bg_value = frame
					.bg
					.get(&Position { x, y })
					.copied()
					.unwrap_or_default();

				let bg_value = world.adjusted_color(bg_value);

				print!("{:02x}", bg_value);
			}
			println!();
		}

		println!("\n{}\n", world.tick_str());
		print!("\n\n\n");
	}
}
