use std::ops::Deref;

use anyhow::{Result, anyhow};

pub struct PeripheralSet<P> {
	peripherals: Vec<Peripheral<P>>,
	reversed: bool,
}

#[derive(Clone)]
pub struct Peripheral<P> {
	peripheral_type: P,
	bit_count: u32,
}

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

impl<P> PeripheralSet<P>
where
	P: PartialEq + Eq + PartialOrd + Ord + PeripheralType,
{
	pub fn new(peripherals: Vec<Peripheral<P>>, reversed: bool) -> Result<Self> {
		let mut peripherals = Self {
			peripherals,
			reversed,
		};

		peripherals.validate()?;
		peripherals.sort();

		Ok(peripherals)
	}

	fn sort(&mut self) {
		self.peripherals
			.sort_unstable_by(|a, b| a.peripheral_type.cmp(&b.peripheral_type));

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
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum InputType {
	Clock,
	CurrentCell,
	NextCell,
	// todo: implement inputs
	// Memory,
	// Random,
	// Ant,
	// CellChange,
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
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum OutputType {
	// todo: implement outputs
	/// Worker Only
	SetCell,
	/// Worker Only
	ClearCell,
	/// 2 bits rotation + 1 bit velocity
	Direction,
	// SetMemory,
	// EnableMemory,
	// /// Queen Only
	// Hatch,
	// /// Queen Only
	// Kill,
	// Die,
}

impl PeripheralType for OutputType {
	fn cap(&self) -> u32 {
		use OutputType::*;

		match self {
			SetCell => 1, // todo: what color depth per ant?
			ClearCell => 1,
			Direction => 3,
		}
	}
}
