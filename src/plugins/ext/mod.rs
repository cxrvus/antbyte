pub mod term_input;

use crate::{plugins::Plugin, world::config::WorldConfig};

pub trait ExtInput: Plugin {
	fn frame(&mut self, config: &WorldConfig) -> u8 {
		_ = config;
		0
	}
}

pub trait ExtOutput: Plugin {
	fn frame(&mut self, config: &WorldConfig, values: &Vec<u8>) {
		_ = config;
		_ = values;
	}
}

pub struct DefaultExtInput;
pub struct DefaultExtOutput;

impl Plugin for DefaultExtInput {}
impl Plugin for DefaultExtOutput {}

impl ExtInput for DefaultExtInput {}
impl ExtOutput for DefaultExtOutput {}
