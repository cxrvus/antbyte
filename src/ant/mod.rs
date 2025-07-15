pub mod circuit;
pub mod parser;

use crate::util::vec2::Vec2;
use anyhow::{Result, anyhow};
use circuit::Circuit;

#[derive(Default)]
pub struct AntConfig {
	sensors: Sensors,
	actions: Actions,
	config: Circuit,
	is_queen: bool,
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

	pub fn tick(&self) -> Actions {
		let sensor_bits: u32 = self.config.sensors.compact(sensor_config)
		let action_bits = self.config.config.tick(sensor_bits);
		action_bits.into()
	}
}

#[derive(Default, PartialEq, PartialOrd)]
pub struct Sensors {
	Clock: u8,
	NextCell: u8,
	// todo: implement sensor fields
	// Memory: u32,
	// Random: u8,
	// Ant: bool,
	// CellChange: bool,
}

impl Sensors {
	pub fn compact(&self, sensor_config: Self) -> Result<u32> {
		let mut bits = 0u32;
		let mut budget = 32u32;

		Self::insert_bits(&mut bits, &mut budget, self.Clock, sensor_config.Clock, 8)?;
		Self::insert_bits(
			&mut bits,
			&mut budget,
			self.NextCell,
			sensor_config.NextCell,
			8,
		)?;

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

#[derive(Default, PartialEq, PartialOrd)]
pub struct Actions {
	direction: u8,
	cell_value: u8,
	cell_write: bool,
	// todo: implement action fields
	// memory_value: u8,
	// memory_write: bool,
	// despawn: bool,
	// /// Queen Only
	// spawn: u8,
}

impl From<u32> for Actions {
	fn from(value: u32) -> Self {
		todo!()
	}
}
