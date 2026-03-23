use crate::plugins::render::{DefaultRenderer, Renderer};

pub mod render;

pub struct Plugins {
	pub renderer: Box<dyn Renderer>,
}

impl Default for Plugins {
	fn default() -> Self {
		Self {
			renderer: Box::new(DefaultRenderer),
		}
	}
}
