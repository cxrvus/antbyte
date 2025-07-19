use std::ops::Deref;

use anyhow::{Result, anyhow};

use crate::ant::AntType;

pub struct PeripheralSet<P> {
	peripherals: Vec<Peripheral<P>>,
	reversed: bool,
}

#[derive(Clone)]
pub struct Peripheral<P> {
	peripheral_type: P,
	bit_count: u32,
}

pub type Input = Peripheral<InputType>;
pub type Output = Peripheral<OutputType>;

impl<P> Peripheral<P>
where
	P: PeripheralType,
{
	pub fn new(peripheral_type: P, bit_count: u32) -> Result<Self> {
		let peripheral = Self {
			peripheral_type,
			bit_count,
		};

		peripheral.validate()?;

		Ok(peripheral)
	}

	pub fn validate(&self) -> Result<()> {
		let (bit_count, cap) = (self.bit_count, self.peripheral_type.cap());

		if bit_count > cap {
			Err(anyhow!("bit count exceeding cap: {bit_count} > {cap}"))
		} else {
			Ok(())
		}
	}

	pub fn peripheral_type(&self) -> &P {
		&self.peripheral_type
	}

	pub fn bit_count(&self) -> u32 {
		self.bit_count
	}
}

impl PeripheralSet<InputType> {
	pub fn inputs(peripherals: Vec<Peripheral<InputType>>) -> Result<Self> {
		Self::new(peripherals, true)
	}
}

impl PeripheralSet<OutputType> {
	pub fn outputs(peripherals: Vec<Peripheral<OutputType>>) -> Result<Self> {
		Self::new(peripherals, false)
	}
}

impl<P> PeripheralSet<P>
where
	P: PartialEq + Eq + PartialOrd + Ord + PeripheralType,
{
	fn new(peripherals: Vec<Peripheral<P>>, reversed: bool) -> Result<Self> {
		let mut peripherals = Self {
			peripherals,
			reversed,
		};

		peripherals.validate()?;
		peripherals.normalize();

		Ok(peripherals)
	}

	fn normalize(&mut self) {
		// remove empty peripherals
		let mut peripherals: Vec<&Peripheral<P>> = self
			.peripherals
			.iter()
			.filter(|x| x.bit_count > 0)
			.collect();

		// sort
		peripherals.sort_unstable_by(|a, b| a.peripheral_type.cmp(&b.peripheral_type));

		if self.reversed {
			self.peripherals.reverse();
		}
	}

	const CAPACITY: u32 = 32;

	pub fn validate(&self) -> Result<()> {
		let bit_count_total = self.iter().map(|p| p.bit_count).sum::<u32>();

		if bit_count_total > Self::CAPACITY {
			Err(anyhow!("maximum peripheral bit capacity exceeded"))
		} else if self.iter().any(|p| {
			self.iter()
				.filter(|q| p.peripheral_type == q.peripheral_type)
				.count() > 1
		}) {
			Err(anyhow!("duplicate peripherals found"))
		} else {
			Ok(())
		}
	}

	// todo: implement CRUD
}

impl<P> Deref for PeripheralSet<P> {
	type Target = Vec<Peripheral<P>>;

	fn deref(&self) -> &Self::Target {
		&self.peripherals
	}
}

pub trait PeripheralType {
	fn cap(&self) -> u32;
	fn is_legal(&self, ant_type: &AntType) -> bool {
		_ = ant_type;
		true
	}
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InputType {
	Clock,
	CurrentCell,
	NextCell,
	// todo: implement inputs
	Memory,
	Random,
	Ant,
}

impl PeripheralType for InputType {
	fn cap(&self) -> u32 {
		8

		// use InputType::*;
		// match self {
		// 	_ => 8,
		// }
	}
}

// TODO: output order is extremely important
// todo: check queen / worker privileges using specified Peripheral sets
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutputType {
	// todo: implement outputs
	/// Worker Only
	SetCell,
	/// Worker Only
	ClearCell,
	/// 2 bits rotation + 1 bit velocity
	Direction,
	SetMemory,
	EnableMemory,
	/// Queen Only
	Hatch,
	/// Queen Only
	Kill,
	Die,
}

impl PeripheralType for OutputType {
	fn cap(&self) -> u32 {
		use OutputType::*;

		match self {
			SetCell => 4,
			ClearCell => 1,
			Direction => 3,
			SetMemory => 8,
			EnableMemory => 1,
			Hatch => 4,
			Kill => 1,
			Die => 1,
		}
	}

	fn is_legal(&self, ant_type: &AntType) -> bool {
		match (ant_type, self) {
			(AntType::Queen, Self::SetCell | Self::ClearCell) => false,
			// (AntType::Worker, Self::Hatch | Self::Kill) => false,
			_ => true,
		}
	}
}
