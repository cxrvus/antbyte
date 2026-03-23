pub mod term_render;

use crate::{util::vec2::Vec2u, world::World};

pub trait Renderer {
	fn open(&mut self);
	fn render(&mut self, world: &World);
	fn close(&self);
}

pub struct DefaultRenderer;

impl Renderer for DefaultRenderer {
	fn open(&mut self) {}
	fn close(&self) {}

	fn render(&mut self, world: &World) {
		let cells = world.cells.clone();

		for y in 0..cells.height {
			for x in 0..cells.width {
				let cell_value = cells.at(&Vec2u { x, y }.sign()).unwrap().value;
				print!("{:02x}", cell_value);
			}
			println!();
		}

		println!("\n{}\n", world.tick_str());
		print!("\n\n\n");
	}
}
