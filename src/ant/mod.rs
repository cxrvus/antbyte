pub mod circuit;
pub mod parser;

use crate::util::vec2::Vec2;
use anyhow::{Result, anyhow};
use circuit::Circuit;

#[derive(Default)]
pub enum AntType {
	#[default]
	Worker,
	Queen,
}

#[derive(Default)]
pub struct AntConfig {
	inputs: Vec<Peripheral<InputType>>,
	outputs: Vec<Peripheral<OutputType>>,
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

	pub fn tick(&self) -> Action {
		let sensor_bits: u32 = self.config.inputs.compact(sensor_config);
		let action_bits = self.config.circuit.tick(sensor_bits);
		action_bits.into()
	}
}

pub struct Peripheral<P> {
	peripheral: P,
	bit_count: u32,
}

#[derive(PartialEq, PartialOrd)]
pub enum InputType {
	Clock,
	CurrentCell,
	NextCell,
	// todo: implement sensor fields
	// Memory,
	// Random,
	// Ant,
	// CellChange,
}

impl Sensor {
	pub fn compact(&self, sensor_config: Self) -> Result<u32> {
		let mut bits = 0u32;
		let mut budget = 32u32;

		// Self::insert_bits(&mut bits, &mut budget, self.Clock, sensor_config.Clock, 8)?;
		// Self::insert_bits(
		// 	&mut bits,
		// 	&mut budget,
		// 	self.NextCell,
		// 	sensor_config.NextCell,
		// 	8,
		// )?;

		Ok(bits)
	}

	fn insert_bits(
		target_bits: &mut u32,
		bit_budget: &mut u32,
		value: impl Into<u32>,
		bit_count: impl Into<u32>,
		bit_limit: u32,
	) -> Result<()> {
		let bit_count: u32 = bit_count.into();
		let value: u32 = value.into();

		if *bit_budget < bit_count {
			return Err(anyhow!("bit budget exceeded"));
		} else {
			*bit_budget -= bit_count;
		}

		if bit_count == 0 {
			Ok(())
		} else if bit_count > bit_limit {
			Err(anyhow!(
				"maximum number of bits reached for that field (limit = {})",
				bit_limit
			))
		} else {
			*target_bits |= value & 1u32.unbounded_shl(31).wrapping_sub(1);
			Ok(())
		}
	}
}

// todo: check queen / worker privileges using specified Peripheral sets
#[derive(PartialEq, PartialOrd)]
pub enum OutputType {
	// todo: implement action fields
	/// 2 bits rotation + 1 bit velocity
	Direction,
	/// Worker Only
	SetCell,
	/// Worker Only
	ClearCell,
	// SetMemory,
	// EnableMemory,
	// /// Queen Only
	// Hatch,
	// /// Queen Only
	// Kill,
}
