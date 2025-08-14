pub mod peripherals;
pub mod world;

pub use world::parser::compiler;

use self::peripherals::{InputType, OutputType, PeripheralSet, PeripheralType};

use crate::{
	circuit::Circuit,
	util::vec2::{Vec2, Vec2u},
};

use anyhow::{Result, anyhow};

// idea: add Cycle & Wrap
#[rustfmt::skip]
#[derive(Debug)]
pub enum BorderMode { Collide, Despawn }

#[rustfmt::skip]
#[derive(Debug)]
pub enum StartingPos { TopLeft, Center }

#[derive(Clone, Copy, Default)]
pub struct Ant {
	pub archetype: u32,
	pub alive: bool,
	pub pos: Vec2u,
	/// cardinal direction - number between 0 and 3
	pub dir: u8,
	pub age: u32,
	pub memory: u32,
}

// todo: move methods to world
impl Ant {
	pub fn new(archetype: u32) -> Self {
		Self {
			archetype,
			alive: true,
			..Default::default()
		}
	}

	pub fn die(&mut self) {
		self.alive = false;
	}

	pub fn get_dir_vec(&self) -> Vec2 {
		assert!(self.dir < 4);
		Vec2::cardinal()[self.dir as usize]
	}

	pub fn set_dir(&mut self, dir: u8) {
		self.dir = dir % 4;
	}

	pub fn flip_dir(&mut self) {
		self.set_dir(self.dir + 2);
	}
}

#[rustfmt::skip]
#[derive(Debug, Clone)]
pub enum AntType { Worker, Queen }

#[derive(Debug)]
pub struct Archetype {
	pub ant_type: AntType,
	pub circuit: Circuit,
	pub inputs: PeripheralSet<InputType>,
	pub outputs: PeripheralSet<OutputType>,
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
			.find(|x| !x.peripheral_type().is_legal(&self.ant_type))
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
