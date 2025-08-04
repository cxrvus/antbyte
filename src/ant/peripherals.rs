use super::AntType;

use anyhow::{Ok, Result, anyhow};
use regex::Regex;
use std::ops::Deref;

#[derive(Debug)]
pub struct PeripheralSet<P> {
	peripherals: Vec<Peripheral<P>>,
	is_inputs: bool,
}

#[derive(Clone, Debug)]
pub struct Peripheral<P> {
	peripheral_type: P,
	bit: u32,
}

pub type Input = Peripheral<InputType>;
pub type Output = Peripheral<OutputType>;

impl<P> Peripheral<P>
where
	P: PeripheralType,
{
	pub fn new(peripheral_type: P, bit: u32) -> Result<Self> {
		let peripheral = Self {
			peripheral_type,
			bit,
		};

		peripheral.validate()?;

		Ok(peripheral)
	}

	pub fn validate(&self) -> Result<()> {
		let (bit_index, cap) = (self.bit, self.peripheral_type.cap());

		if bit_index > cap {
			Err(anyhow!("bit index exceeding cap: {bit_index} > {cap}"))
		} else {
			Ok(())
		}
	}

	const PERIPH_PTN: &str = r"^([A-Z]{1,4})([0-9a-f])?$";

	pub fn from_ident(ident: String) -> Result<Self> {
		let re = Regex::new(Self::PERIPH_PTN).unwrap();

		// let (periph_string, bit_index) =
		let captures = re
			.captures(&ident)
			.ok_or(anyhow!("'{ident}' is not a valid peripheral"))?;

		let type_ident = captures.get(1).unwrap().as_str();

		let p_type = P::from_ident(&type_ident.to_ascii_lowercase())
			.ok_or(anyhow!("'{type_ident}' is not a valid peripheral type"))?;

		let bit_index = captures
			.get(2)
			.map(|m| u32::from_str_radix(m.as_str(), 16).unwrap());

		let cap = p_type.cap();

		if let Some(bit_index) = bit_index {
			if cap == 1 {
				return Err(anyhow!(
					"may not have a bit index in one-bit peripherals\n(in '{ident}')"
				));
			} else if bit_index >= cap {
				return Err(anyhow!(
					"bit index may not exceed peripheral bit capacity:\n{bit_index} >= {cap}\n(in '{ident}')"
				));
			}
		};

		Ok(Self {
			peripheral_type: p_type,
			bit: bit_index.unwrap_or_default(),
		})
	}

	pub fn peripheral_type(&self) -> &P {
		&self.peripheral_type
	}

	pub fn bit(&self) -> u32 {
		self.bit
	}
}

impl PeripheralSet<InputType> {
	pub fn inputs(peripherals: Vec<Peripheral<InputType>>) -> Result<Self> {
		Self::from_spec(peripherals, true)
	}
}

impl PeripheralSet<OutputType> {
	pub fn outputs(peripherals: Vec<Peripheral<OutputType>>) -> Result<Self> {
		Self::from_spec(peripherals, false)
	}
}

impl<P> PeripheralSet<P>
where
	P: PartialEq + Eq + PartialOrd + Ord + PeripheralType,
{
	fn from_spec(peripheral_spec: Vec<Peripheral<P>>, is_inputs: bool) -> Result<Self> {
		let mut peripherals = Self {
			peripherals: peripheral_spec,
			is_inputs,
		};

		peripherals.validate()?;
		peripherals.normalize();

		Ok(peripherals)
	}

	fn normalize(&mut self) {
		// remove empty peripherals
		let mut peripherals: Vec<&Peripheral<P>> =
			self.peripherals.iter().filter(|x| x.bit > 0).collect();

		// sort
		peripherals.sort_unstable_by(|a, b| a.peripheral_type.cmp(&b.peripheral_type));

		if self.is_inputs {
			self.peripherals.reverse();
		}
	}

	const CAPACITY: u32 = 32;

	pub fn validate(&self) -> Result<()> {
		let bit_count_total = self.iter().map(|p| p.bit).sum::<u32>();

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

	/// takes a vector with all the used peripherals and returns a Set
	/// containing the ones with highest bits respectively
	pub fn from_used(used_peripherals: Vec<Peripheral<P>>, is_inputs: bool) -> Result<Self> {
		let mut specs: Vec<Peripheral<P>> = vec![];

		used_peripherals.into_iter().for_each(|peripheral| {
			if let Some(collision_index) = specs.iter().position(|x| {
				x.peripheral_type() == peripheral.peripheral_type() && x.bit() < peripheral.bit()
			}) {
				specs[collision_index] = peripheral;
			} else {
				specs.push(peripheral);
			}
		});

		// increase each bit specifier by 1,
		// because we need the capacity, not just the highest bit index
		specs.iter_mut().for_each(|spec| spec.bit += 1);

		Self::from_spec(specs, is_inputs)
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

pub trait PeripheralType: Sized {
	fn cap(&self) -> u32;
	fn from_ident(ident: &str) -> Option<Self>;
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

	fn from_ident(ident: &str) -> Option<Self> {
		use InputType::*;

		match ident {
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
	MemoryClear,
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
			MemoryClear => 1,
			Spawn => 4,
			Kill => 1,
			Die => 1,
		}
	}

	fn is_legal(&self, ant_type: &AntType) -> bool {
		!match ant_type {
			AntType::Worker => matches!(Self::Spawn, Self::Kill),
			AntType::Queen => matches!(Self::CellWrite, Self::CellClear),
		}
	}

	fn from_ident(ident: &str) -> Option<Self> {
		use OutputType::*;

		match ident {
			"cx" => Some(CellWrite),
			"cq" => Some(CellClear),
			"d" => Some(Direction),
			"mx" => Some(MemoryWrite),
			"mq" => Some(MemoryClear),
			"spx" => Some(Spawn),
			"kill" => Some(Kill),
			"die" => Some(Die),
			_ => None,
		}
	}
}
