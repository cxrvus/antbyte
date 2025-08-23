pub mod peripherals;
pub mod world;

pub use world::parser::compiler;

use self::peripherals::{InputType, OutputType, PeripheralSet, PeripheralType};

use crate::{
	truth_table::TruthTable,
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
	pub behavior: u32,
	pub alive: bool,
	pub pos: Vec2u,
	/// cardinal direction - number between 0 and 3
	pub dir: u8,
	pub age: u32,
	pub memory: u32,
}

// todo: move methods to world
impl Ant {
	pub fn new(behavior: u32) -> Self {
		Self {
			behavior,
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
pub struct Behavior {
	pub ant_type: AntType,
	pub truth_table: TruthTable,
	pub inputs: PeripheralSet<InputType>,
	pub outputs: PeripheralSet<OutputType>,
}

impl Behavior {
	pub fn new(
		ant_type: AntType,
		truth_table: TruthTable,
		inputs: PeripheralSet<InputType>,
		outputs: PeripheralSet<OutputType>,
	) -> Result<Self> {
		let behavior = Self {
			ant_type,
			truth_table,
			inputs,
			outputs,
		};

		behavior.validate()?;

		Ok(behavior)
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
