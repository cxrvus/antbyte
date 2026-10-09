use crate::{
	ant::{
		pin::{IoType, Pin},
		sub_pin::SubPin,
	},
	truth_table::TruthTable,
	util::find_dupe,
};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

pub const MAX_INPUTS: usize = 12;
pub const MAX_OUTPUTS: usize = 16;

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(try_from = "BehaviorDTO", into = "BehaviorDTO")]
pub struct Behavior {
	pub name: String,
	pub logic: TruthTable,
	pub inputs: Vec<SubPin>,
	pub outputs: Vec<SubPin>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BehaviorDTO {
	name: String,
	logic: Vec<u32>,
	inputs: Vec<SubPin>,
	outputs: Vec<SubPin>,
}

impl TryFrom<BehaviorDTO> for Behavior {
	type Error = anyhow::Error;

	fn try_from(value: BehaviorDTO) -> Result<Self, Self::Error> {
		let logic = TruthTable::new(value.inputs.len(), value.outputs.len(), value.logic)?;
		Behavior::new(value.name, logic, value.inputs, value.outputs)
	}
}

impl From<Behavior> for BehaviorDTO {
	fn from(value: Behavior) -> Self {
		Self {
			name: value.name,
			logic: value.logic.entries(),
			inputs: value.inputs,
			outputs: value.outputs,
		}
	}
}

impl Behavior {
	pub fn new(
		name: String,
		truth_table: TruthTable,
		inputs: Vec<SubPin>,
		outputs: Vec<SubPin>,
	) -> Result<Self> {
		if inputs.len() > MAX_INPUTS {
			return Err(anyhow!(
				"may not have more than {MAX_INPUTS} inputs, got {}\n{:?}:\n",
				inputs.len(),
				inputs
			));
		} else if outputs.len() > MAX_OUTPUTS {
			return Err(anyhow!(
				"may not have more than {MAX_OUTPUTS} inputs, got {}\n{:?}:\n",
				outputs.len(),
				outputs
			));
		}

		Self::validate_pins(&inputs, IoType::Input)?;
		Self::validate_pins(&outputs, IoType::Output)?;

		Ok(Self {
			logic: truth_table,
			name,
			inputs,
			outputs,
		})
	}

	pub fn validate_pins(pins: &Vec<SubPin>, io_type: IoType) -> Result<()> {
		if let Some(dupe) = find_dupe(pins) {
			Err(anyhow!("found duplicate pin in Behavior: {dupe:?}"))
		} else {
			for pin in pins {
				pin.validate(&io_type)?;
			}

			Ok(())
		}
	}

	pub fn pin_mask(&self, pin: Pin) -> u8 {
		let mut mask = 0;

		for sub_pin in &self.outputs {
			if pin == sub_pin.pin {
				mask |= 1 << sub_pin.line();
			}
		}

		mask
	}
}
