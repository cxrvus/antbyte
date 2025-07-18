pub mod circuit;
pub mod parser;
pub mod peripherals;

use std::{ops::Deref, rc::Rc};

use crate::{
	ant::peripherals::*,
	util::vec2::{Vec2, Vec2u},
	world::World,
};
use circuit::Circuit;

#[derive(Default)]
pub enum AntType {
	#[default]
	Worker,
	Queen,
}

#[derive(Default)]
pub struct Archetype {
	inputs: PeripheralSet<InputType>,
	outputs: PeripheralSet<OutputType>,
	circuit: Circuit,
	ant_type: AntType,
}

#[derive(Default, Clone)]
pub struct Ant {
	id: usize,
	archetype: Rc<Archetype>,
	pos: Vec2u,
	dir: Vec2,
	moving: bool,
	age: u32,
	memory: u32,
}

impl Ant {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn tick(&self, world: &mut World) {
		let Archetype {
			inputs,
			outputs,
			circuit,
			..
		} = self.archetype.deref();

		let mut condensed_input = 0u32;

		for input in inputs.iter() {
			let Peripheral {
				peripheral,
				bit_count,
			} = input;

			use InputType::*;

			// getting the input value
			let input_value: u32 = match peripheral {
				Clock => self.age % 0xff,
				CurrentCell => (*world.state.cells.at(&self.pos.sign()).unwrap()).into(),
				NextCell => (*world
					.state
					.cells
					// fixme: account for out-of-bounds
					.at(&(self.pos.sign() + self.dir))
					.unwrap())
				.into(),
			};

			// condensing the input into a u32
			let masked_input_value = input_value & 1u32.unbounded_shl(*bit_count).wrapping_sub(1);
			condensed_input <<= bit_count;
			condensed_input |= masked_input_value;
		}

		// calculating the output
		let mut condensed_output = circuit.tick(condensed_input);

		for output in outputs.iter() {
			let Peripheral {
				peripheral,
				bit_count,
			} = output;

			use OutputType::*;

			let output_value = condensed_output & 1u32.unbounded_shl(*bit_count).wrapping_sub(1);

			match peripheral {
				Direction => todo!(),
				SetCell => todo!(),
				ClearCell => todo!(),
			};

			condensed_output >>= bit_count;
		}
	}
}
