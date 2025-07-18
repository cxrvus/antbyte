use std::ops::Deref;

use anyhow::{Error, Result, anyhow};

#[derive(Default)]
pub struct PeripheralSet<P>(Vec<Peripheral<P>>);

#[derive(Clone, Default)]
pub struct Peripheral<P> {
	pub peripheral: P,
	pub bit_count: u32,
}

impl<P> PeripheralSet<P> {
	const CAPACITY: u32 = 32;

	pub fn validate_capacity(&self) -> Result<()> {
		let bit_count = self.0.iter().map(|p| p.bit_count).sum::<u32>();

		if bit_count > Self::CAPACITY {
			Err(anyhow!("maximum peripheral bit capacity exceeded"))
		} else {
			Ok(())
		}
	}

	// todo: implement CRUD
}

impl<P> Deref for PeripheralSet<P> {
	type Target = Vec<Peripheral<P>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<P> TryFrom<Vec<Peripheral<P>>> for PeripheralSet<P>
where
	P: PartialEq + Eq + PartialOrd + Ord + Default,
{
	type Error = Error;

	fn try_from(peripherals: Vec<Peripheral<P>>) -> Result<Self> {
		let mut peripherals = peripherals;
		peripherals.sort_unstable_by(|a, b| a.peripheral.cmp(&b.peripheral));
		let peripherals = Self(peripherals);

		if peripherals.0.iter().any(|p| {
			peripherals
				.0
				.iter()
				.filter(|q| p.peripheral == q.peripheral)
				.count() > 1
		}) {
			Err(anyhow!("duplicate peripherals found"))
		} else {
			Ok(peripherals)
		}
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
