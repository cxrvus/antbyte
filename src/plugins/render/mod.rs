pub mod term_render;

use crate::world::World;

pub trait Renderer {
	fn open(&mut self);
	fn render(&mut self, world: &World);
	fn close(&self);
}
