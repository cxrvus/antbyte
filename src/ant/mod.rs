pub mod circuit;
pub mod parser;
pub mod peripherals;

use crate::{
	ant::peripherals::*,
	util::vec2::{Vec2, Vec2u},
	world::{BorderMode, World},
};
use anyhow::{Result, anyhow};
use circuit::Circuit;

#[derive(Debug)]
pub enum AntType {
	Worker,
	Queen,
}

pub struct Archetype {
	ant_type: AntType,
	circuit: Circuit,
	inputs: PeripheralSet<InputType>,
	outputs: PeripheralSet<OutputType>,
}

impl Archetype {
	pub fn new(
		ant_type: AntType,
		circuit: Circuit,
		inputs: PeripheralSet<InputType>,
		outputs: PeripheralSet<OutputType>,
	) -> Result<Self> {
		let archetype = Self {
			ant_type,
			circuit,
			inputs,
			outputs,
		};

		archetype.validate()?;

		Ok(archetype)
	}

	pub fn validate(&self) -> Result<()> {
		if let Some(x) = self
			.outputs
			.iter()
			.find(|x| x.peripheral_type().is_legal(&self.ant_type))
		{
			Err(anyhow!(
				"illegal {:?} for {:?}",
				x.peripheral_type(),
				self.ant_type
			))
		} else {
			Ok(())
		}
	}
}

#[derive(Clone, Default)]
pub struct Register {
	current: u32,
	next: u32,
}

impl Register {
	fn overwrite(&mut self) {
		self.current = self.next;
	}
}

#[derive(Clone, Default)]
pub struct Ant {
	archetype: usize,
	alive: bool,
	pos: Vec2u,
	/// cardinal direction - number between 0 and 3
	dir: u8,
	age: u32,
	memory: Register,
}

impl Ant {
	pub fn new(archetype: usize) -> Self {
		Self {
			archetype,
			alive: true,
			..Default::default()
		}
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

		if world.cells.in_bounds(&new_pos) {
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
			if !world.ants.iter().any(|ant| ant.pos == new_pos) {
				self.pos = new_pos;
			}
		} else if let BorderMode::Despawn = world.border_mode() {
			self.die();
		}
	}

	fn get_target_ant<'a>(&self, world: &'a mut World) -> Option<&'a mut Ant> {
		let pos = self.next_pos(world)?.unsign().unwrap();
		world.ants.iter_mut().find(|ant| ant.pos == pos)
	}

	pub fn tick(&mut self, world: &mut World) {
		let world_image = world.clone();

		let Archetype {
			inputs,
			outputs,
			circuit,
			..
		} = world_image
			.get_archetype(self.archetype)
			.expect("invalid Archetype ID");

		let mut condensed_input = 0u32;

		for input in inputs.iter() {
			use InputType::*;

			// getting the input value
			let input_value: u32 = match input.peripheral_type() {
				Clock => self.age % 0x100,
				CurrentCell => (*world.cells.at(&self.pos.sign()).unwrap()).into(),
				NextCell => self
					.next_pos(world)
					.map(|pos| *world.cells.at(&pos).unwrap())
					.unwrap_or(0u8)
					.into(),
				Memory => self.memory.current,
				Random => todo!(),
				Ant => self.get_target_ant(world).is_some().into(),
			};

			// condensing the input values into a single u32 value
			let bit_count = input.bit_count();
			let mask = 1u32.unbounded_shl(bit_count).wrapping_sub(1);
			let masked_input_value = input_value & mask;
			condensed_input <<= bit_count;
			condensed_input |= masked_input_value;
		}

		// calculating the output
		let mut condensed_output = circuit.tick(condensed_input);

		for output in outputs.iter() {
			use OutputType::*;

			// inflating the output bits into multiple u32 values
			let bit_count = output.bit_count();
			let mask = 1u32.unbounded_shl(bit_count).wrapping_sub(1);
			let value = condensed_output & mask;

			match output.peripheral_type() {
				Direction => {
					let moving = value & 1 == 1;
					let rotations = (value >> 1) as u8;
					self.set_dir(self.dir + rotations);

					if moving {
						self.move_tick(world);
					}
				}
				SetCell if value != 0 => world.cells.set_at(&self.pos.sign(), 1),
				ClearCell if value != 0 => world.cells.set_at(&self.pos.sign(), 0),
				SetMemory => self.memory.next = value,
				EnableMemory => self.memory.overwrite(),
				Hatch => todo!(),
				Kill => {
					if let Some(ant) = self.get_target_ant(world) {
						ant.die();
					}
				}
				Die => self.die(),
				_ => {}
			};

			condensed_output >>= bit_count;
		}
	}
}
