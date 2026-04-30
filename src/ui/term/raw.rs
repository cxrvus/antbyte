use crate::{util::vec2::Position, world::World};

fn render(world: &World) {
	let cells = world.cells.clone();

	for y in 0..cells.height {
		for x in 0..cells.width {
			let cell_value = cells.at(Position { x, y }).unwrap().value;
			let cell_value = world.adjusted_color(cell_value);
			print!("{:02x}", cell_value);
		}
		println!();
	}

	println!("\n{}\n", world.tick_str());
	print!("\n\n\n");
}
