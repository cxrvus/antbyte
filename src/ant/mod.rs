pub mod circuit;
pub mod controller;
pub mod parser;

use crate::{ant::controller::*, util::vec2::Vec2};
use circuit::Circuit;

#[derive(Default)]
pub enum AntType {
	#[default]
	Worker,
	Queen,
}

#[derive(Default)]
pub struct AntConfig {
	inputs: PeripheralSet<InputType>,
	outputs: PeripheralSet<OutputType>,
	circuit: Circuit,
	ant_type: AntType,
}

#[derive(Default)]
pub struct Ant {
	config: AntConfig,
	age: u32,
	memory: u32,
	dir: Vec2,
}

impl Ant {
	pub fn new(config: AntConfig) -> Self {
		Self {
			config,
			..Default::default()
		}
	}

	pub fn tick(&self) -> u32 {
		let input_bits: u32 = self.config.inputs.compact();
		let output_bits = self.config.circuit.tick(input_bits);
		output_bits.into()
	}
}
