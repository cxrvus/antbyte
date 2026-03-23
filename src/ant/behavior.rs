use crate::{
	ant::pin::{IoType, SubPin},
	truth_table::TruthTable,
	util::find_dupe,
};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(try_from = "BehaviorJSON", into = "BehaviorJSON")]
pub struct Behavior {
	pub name: String,
	pub logic: TruthTable,
	pub inputs: Vec<SubPin>,
	pub outputs: Vec<SubPin>,
}

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
struct BehaviorJSON {
	name: String,
	logic: Vec<u32>,
	inputs: Vec<SubPin>,
	outputs: Vec<SubPin>,
}

impl TryFrom<BehaviorJSON> for Behavior {
	type Error = String;

	fn try_from(value: BehaviorJSON) -> std::result::Result<Self, Self::Error> {
		let logic = TruthTable::new(value.inputs.len(), value.outputs.len(), value.logic)
			.map_err(|e| e.to_string())?;

		Behavior::new(value.name, logic, value.inputs, value.outputs).map_err(|e| e.to_string())
	}
}

impl From<Behavior> for BehaviorJSON {
	fn from(value: Behavior) -> Self {
		Self {
			name: value.name,
			logic: value.logic.entries().clone(),
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
		if inputs.len() > 8 {
			return Err(anyhow!(
				"may not have more than 8 inputs, got {}\n{:?}:\n",
				inputs.len(),
				inputs
			));
		} else if outputs.len() > 32 {
			return Err(anyhow!(
				"may not have more than 32 inputs, got {}\n{:?}:\n",
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
}
