pub mod term_render;

use crate::world::World;

pub trait Renderer {
	fn render(&mut self, world: &World);
	fn end(&self);
}
