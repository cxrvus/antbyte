pub mod circuit;
pub mod parser;
pub mod peripherals;

use crate::{
	ant::peripherals::*,
	util::vec2::{Vec2, Vec2u},
	world::{BorderMode, World},
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
	instance_id: usize,
	archetype_id: usize,
	alive: bool,
	pos: Vec2u,
	/// cardinal direction - number between 0 and 3
	dir: u8,
	moving: bool,
	age: u32,
	memory: u32,
}

impl Ant {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn id(&self) -> usize {
		self.instance_id
	}

	pub fn die(&mut self) {
		self.alive = false;
	}

	pub fn is_alive(&self) -> bool {
		self.alive
	}

	pub fn get_dir_vec(&self) -> Vec2 {
		assert!(self.dir < 4);
		Vec2::cardinal()[self.dir as usize]
	}

	pub fn set_dir(&mut self, dir: u8) {
		self.dir = (self.dir + dir) % 4;
	}

	pub fn next_pos(&self, world: &World) -> Option<Vec2> {
		let (pos, dir) = (self.pos.sign(), self.get_dir_vec());
		let new_pos = pos + dir;

		if world.state.cells.in_bounds(&new_pos) {
			Some(new_pos)
		} else {
			use BorderMode::*;

			match world.border_mode() {
				Collide | Despawn => None,
			}
		}
	}

	pub fn move_tick(&mut self, world: &World) {
		if let Some(new_pos) = self.next_pos(world) {
			let new_pos = new_pos.unsign().unwrap();

			// ant collision check
			if !world.state.ants.iter().any(|ant| ant.pos == new_pos) {
				self.pos = new_pos;
			}
		} else if let BorderMode::Despawn = world.border_mode() {
			self.die();
		}
	}

	pub fn tick(&mut self, world: &mut World) {
		let world_image = world.clone();

		let Archetype {
			inputs,
			outputs,
			circuit,
			..
		} = world_image
			.get_archetype(self.archetype_id)
			.expect("invalid Archetype ID");

		let mut condensed_input = 0u32;

		for input in inputs.iter() {
			let Peripheral {
				peripheral,
				bit_count,
			} = input;

			use InputType::*;

			// getting the input value
			let input_value: u32 = match peripheral {
				Clock => self.age % 0x100,
				CurrentCell => (*world.state.cells.at(&self.pos.sign()).unwrap()).into(),
				NextCell => self
					.next_pos(world)
					.map(|pos| *world.state.cells.at(&pos).unwrap())
					.unwrap_or(0u8)
					.into(),
			};

			// condensing the input values into a single u32 value
			let mask = 1u32.unbounded_shl(*bit_count).wrapping_sub(1);
			let masked_input_value = input_value & mask;
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

			// inflating the output bits into multiple u32 values
			let mask = 1u32.unbounded_shl(*bit_count).wrapping_sub(1);
			let output_value = condensed_output & mask;

			match peripheral {
				Direction => {
					let moving = output_value & 1 == 1;
					let rotations = (output_value >> 1) as u8;
					self.set_dir(self.dir + rotations);

					if moving {
						self.move_tick(world);
					}
				}
				SetCell if output_value != 0 => {
					world.state.cells.set_at(&self.pos.sign(), 1);
				}
				ClearCell if output_value != 0 => {
					world.state.cells.set_at(&self.pos.sign(), 0);
				}
				_ => {}
			};

			condensed_output >>= *bit_count;
		}
	}
}
