use anyhow::{Result, anyhow};

use crate::ant::{
	circuit::Circuit,
	peripherals::{InputType, OutputType, PeripheralSet, PeripheralType},
};

#[derive(Debug)]
pub enum AntType {
	Worker,
	Queen,
}

pub struct Archetype {
	pub(super) ant_type: AntType,
	pub(super) circuit: Circuit,
	pub(super) inputs: PeripheralSet<InputType>,
	pub(super) outputs: PeripheralSet<OutputType>,
}

impl Archetype {
	pub fn new(
		ant_type: AntType,
		circuit: Circuit,
		inputs: PeripheralSet<InputType>,
		outputs: PeripheralSet<OutputType>,
	) -> Result<Self> {
		let archetype = Self {
			ant_type,
			circuit,
			inputs,
			outputs,
		};

		archetype.validate()?;

		Ok(archetype)
	}

	pub fn validate(&self) -> Result<()> {
		if let Some(x) = self
			.outputs
			.iter()
			.find(|x| x.peripheral_type().is_legal(&self.ant_type))
		{
			Err(anyhow!(
				"illegal {:?} for {:?}",
				x.peripheral_type(),
				self.ant_type
			))
		} else {
			Ok(())
		}
	}
}

#[derive(Clone, Default)]
pub struct Register {
	pub current: u32,
	pub next: u32,
}

impl Register {
	pub fn overwrite(&mut self) {
		self.current = self.next;
	}
}
