pub mod circuit;
pub mod parser;

use crate::util::vec2::Vec2;
use anyhow::{Error, Result, anyhow};
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

#[derive(Default)]
pub struct PeripheralSet<P>(Vec<Peripheral<P>>)
where
	P: PartialEq + PartialOrd + Default;

#[derive(Default)]
pub struct Peripheral<P>
where
	P: PartialEq + PartialOrd + Default,
{
	peripheral: P,
	/// can be used as either the actual value (Output) or as the desired bit count (Input)
	value: u32,
}

impl<P> PeripheralSet<P>
where
	P: PartialEq + PartialOrd + Default,
{
	fn validate_capacity(&self) -> bool {
		self.0.iter().map(|p| p.value).sum::<u32>() <= 32
	}

	// todo: implement CRUD
}

impl<P> TryFrom<Vec<Peripheral<P>>> for PeripheralSet<P>
where
	P: PartialEq + PartialOrd + Default,
{
	type Error = Error;

	fn try_from(peripherals: Vec<Peripheral<P>>) -> Result<Self> {
		if peripherals.iter().any(|p| {
			peripherals
				.iter()
				.filter(|q| p.peripheral == q.peripheral)
				.count() > 1
		}) {
			Err(anyhow!("duplicate peripherals found"))
		} else {
			Ok(Self(peripherals))
		}
	}
}

impl PeripheralSet<InputType> {
	pub fn compact(&self) -> u32 {
		todo!()
	}
}

impl PeripheralSet<OutputType> {
	pub fn inflate(&self, output: u32) -> Self {
		todo!()
	}
}

#[derive(PartialEq, PartialOrd, Default)]
pub enum InputType {
	#[default]
	Clock,
	CurrentCell,
	NextCell,
	// todo: implement sensor fields
	// Memory,
	// Random,
	// Ant,
	// CellChange,
}

// todo: check queen / worker privileges using specified Peripheral sets
#[derive(PartialEq, PartialOrd, Default)]
pub enum OutputType {
	// todo: implement action fields
	/// 2 bits rotation + 1 bit velocity
	#[default]
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

// todo: recycle --------------------------------------------------------------

struct Sensor_Deprecated;

impl Sensor_Deprecated {
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
