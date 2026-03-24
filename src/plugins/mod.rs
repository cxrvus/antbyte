use crate::{
	plugins::{
		ext::{DefaultExtInput, DefaultExtOutput, ExtInput, ExtOutput},
		render::{DefaultRenderer, Renderer},
	},
	world::config::WorldConfig,
};

pub mod ext;
pub mod render;

#[rustfmt::skip]
pub trait Plugin {
	fn open(&mut self, config: &WorldConfig) { _ = config }
	fn close(&self) {}
}

pub struct PluginSet {
	pub renderer: Box<dyn Renderer>,
	pub ext_input: Box<dyn ExtInput>,
	pub ext_output: Box<dyn ExtOutput>,
}

impl Default for PluginSet {
	fn default() -> Self {
		Self {
			renderer: Box::new(DefaultRenderer),
			ext_input: Box::new(DefaultExtInput),
			ext_output: Box::new(DefaultExtOutput),
		}
	}
}
