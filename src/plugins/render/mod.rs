pub mod term_render;

use crate::{plugins::Plugin, util::vec2::Vec2u, world::World};

pub trait Renderer: Plugin {
	fn render(&mut self, world: &World);
}

pub struct DefaultRenderer;
impl Plugin for DefaultRenderer {}

impl Renderer for DefaultRenderer {
	fn render(&mut self, world: &World) {
		let cells = world.cells.clone();

		for y in 0..cells.height {
			for x in 0..cells.width {
				let cell_value = cells.at(&Vec2u { x, y }.sign()).unwrap().value;
				let cell_value = world.adjusted_color(cell_value);
				print!("{:02x}", cell_value);
			}
			println!();
		}

		println!("\n{}\n", world.tick_str());
		print!("\n\n\n");
	}
}
