use std::ops::Deref;

use anyhow::{Error, Result, anyhow};

use crate::ant::AntType;

#[derive(Debug)]
pub struct PeripheralSet<P> {
	peripherals: Vec<Peripheral<P>>,
	reversed: bool,
}

#[derive(Clone, Debug)]
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

pub const CELL_CAP: u32 = 4;
pub const MEM_CAP: u32 = 16;

pub trait PeripheralType {
	fn cap(&self) -> u32;
	fn from_ident(ident: String) -> Option<impl PeripheralType>;
	fn is_legal(&self, ant_type: &AntType) -> bool {
		_ = ant_type;
		true
	}
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InputType {
	Time,
	Cell,
	CellNext,
	Memory,
	Random,
	Ant,
}

impl PeripheralType for InputType {
	fn cap(&self) -> u32 {
		match self {
			InputType::Time => 8,
			InputType::Cell => CELL_CAP,
			InputType::CellNext => CELL_CAP,
			InputType::Memory => MEM_CAP,
			InputType::Random => 8,
			InputType::Ant => 1,
		}
	}

	fn from_ident(indent: String) -> Option<impl PeripheralType> {
		use InputType::*;
		let value = indent.to_ascii_lowercase();

		match value.as_str() {
			"t" => Some(Time),
			"c" => Some(Cell),
			"cn" => Some(CellNext),
			"m" => Some(Memory),
			"r" => Some(Random),
			"ant" => Some(Ant),
			_ => None,
		}
	}
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutputType {
	/// Worker Only
	CellWrite,
	/// Worker Only
	CellClear,
	// todo: split up into 3 separate bits
	/// 2 bits rotation + 1 bit velocity
	Direction,
	MemoryWrite,
	MemoryEnable,
	/// Queen Only
	Spawn,
	/// Queen Only
	Kill,
	Die,
}

impl PeripheralType for OutputType {
	fn cap(&self) -> u32 {
		use OutputType::*;

		match self {
			CellWrite => CELL_CAP,
			CellClear => 1,
			Direction => 3,
			MemoryWrite => MEM_CAP,
			MemoryEnable => 1,
			Spawn => 4,
			Kill => 1,
			Die => 1,
		}
	}

	fn is_legal(&self, ant_type: &AntType) -> bool {
		match (ant_type, self) {
			(AntType::Queen, Self::CellWrite | Self::CellClear) => false,
			// (AntType::Worker, Self::Hatch | Self::Kill) => false,
			_ => true,
		}
	}

	fn from_ident(indent: String) -> Option<impl PeripheralType> {
		use OutputType::*;
		let value = indent.to_ascii_lowercase();

		match value.as_str() {
			"cx" => Some(CellWrite),
			"cq" => Some(CellClear),
			"dir" => Some(Direction),
			"mx" => Some(MemoryWrite),
			// todo: MemoryClear (MQ) instead of MemoryEnable
			"mm" => Some(MemoryEnable),
			"spx" => Some(Spawn),
			"kill" => Some(Kill),
			"die" => Some(Die),
			_ => None,
		}
	}
}
