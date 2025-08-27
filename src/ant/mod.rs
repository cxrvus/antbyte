pub mod peripherals;
pub mod world;

pub use world::parser::compiler;

use crate::{
	ant::peripherals::{IoType, PeripheralBit},
	truth_table::TruthTable,
	util::{
		find_dupe,
		vec2::{Vec2, Vec2u},
	},
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
	pub behavior: u8,
	pub alive: bool, // todo: deprecate,
	pub pos: Vec2u,
	/// cardinal direction - number between 0 and 3
	pub dir: u8,
	pub moving: bool,
	pub memory: u8,
	pub age: u32,
}

impl Ant {
	pub fn new(behavior: u8) -> Self {
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

#[derive(Debug)]
pub struct Behavior {
	pub truth_table: TruthTable,
	pub inputs: Vec<PeripheralBit>,
	pub outputs: Vec<PeripheralBit>,
}

impl Behavior {
	pub fn new(
		truth_table: TruthTable,
		inputs: Vec<PeripheralBit>,
		outputs: Vec<PeripheralBit>,
	) -> Result<Self> {
		Self::validate_periphs(&inputs, IoType::Input)?;
		Self::validate_periphs(&outputs, IoType::Output)?;

		Ok(Self {
			truth_table,
			inputs,
			outputs,
		})
	}

	pub fn validate_periphs(periphs: &Vec<PeripheralBit>, io_type: IoType) -> Result<()> {
		if let Some(dupe) = find_dupe(periphs) {
			Err(anyhow!("found duplicate peripheral in Behavior: {dupe:?}"))
		} else {
			for periph in periphs {
				periph.validate(&io_type)?;
			}

			Ok(())
		}
	}
}
