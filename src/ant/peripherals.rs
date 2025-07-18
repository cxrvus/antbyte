use std::ops::Deref;

use anyhow::{Result, anyhow};

#[derive(Default)]
pub struct PeripheralSet<P> {
	peripherals: Vec<Peripheral<P>>,
	reversed: bool,
}

#[derive(Clone, Default)]
pub struct Peripheral<P> {
	pub peripheral: P,
	pub bit_count: u32,
}

impl<P> PeripheralSet<P>
where
	P: PartialEq + Eq + PartialOrd + Ord,
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
			.sort_unstable_by(|a, b| a.peripheral.cmp(&b.peripheral));

		if self.reversed {
			self.peripherals.reverse();
		}
	}

	const CAPACITY: u32 = 32;

	pub fn validate(&self) -> Result<()> {
		let bit_count_total = self.iter().map(|p| p.bit_count).sum::<u32>();

		if bit_count_total > Self::CAPACITY {
			Err(anyhow!("maximum peripheral bit capacity exceeded"))
		} else if self
			.iter()
			.any(|p| self.iter().filter(|q| p.peripheral == q.peripheral).count() > 1)
		{
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Default)]
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

// TODO: output order is extremely important
// todo: check queen / worker privileges using specified Peripheral sets
#[derive(PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum OutputType {
	// todo: implement action fields
	/// Worker Only
	SetCell,
	/// Worker Only
	ClearCell,
	/// 2 bits rotation + 1 bit velocity
	#[default]
	Direction,
	// SetMemory,
	// EnableMemory,
	// /// Queen Only
	// Hatch,
	// /// Queen Only
	// Kill,
	// Die,
}
