use anyhow::{Ok, Result, anyhow, bail};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ant::pin::{IoType, Pin};

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubPin {
	pub pin: Pin,
	pub line: u8,
}

impl Serialize for SubPin {
	fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&self.to_ident())
	}
}

impl<'de> Deserialize<'de> for SubPin {
	fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let ident = String::deserialize(deserializer)?;
		Self::from_ident(&ident).map_err(serde::de::Error::custom)
	}
}

impl SubPin {
	pub fn to_ident(&self) -> String {
		let mut ident = self.pin.short_ident().to_owned();

		if self.pin.definition().size > 1 {
			ident.push_str(&format!("{:x}", self.line));
		}

		ident
	}

	pub fn validate(&self, io_type: &IoType) -> Result<()> {
		let definition = self.pin.definition();

		let bit_exceeding_size = self.line >= definition.size;

		let wrong_io_type = match definition.io_type {
			Some(req_io_type) => req_io_type != *io_type,
			None => false,
		};

		if bit_exceeding_size {
			Err(anyhow!("bit index exceeding size: {self:?}",))
		} else if wrong_io_type {
			Err(anyhow!("wrong Input / Output type for Pin: {self:?}",))
		} else {
			Ok(())
		}
	}

	const PIN_PTN: &str = r"^([A-Z_]+)([0-8])?$";

	pub fn from_ident(ident: &str) -> Result<Self> {
		let re = Regex::new(Self::PIN_PTN).unwrap();

		let captures = re
			.captures(ident)
			.ok_or(anyhow!("'{ident}' is not a valid pin"))?;

		let pin_ident = captures.get(1).unwrap().as_str();

		let pin =
			Pin::from_ident(pin_ident).ok_or(anyhow!("'{pin_ident}' is not a valid pin type"))?;

		let bit_index = captures
			.get(2)
			.map(|m| u8::from_str_radix(m.as_str(), 16).unwrap());

		let size = pin.definition().size;

		if let Some(bit_index) = bit_index {
			if size == 1 {
				bail!("may not have a bit index in one-bit pins\n(in '{ident}')");
			} else if bit_index >= size {
				bail!(
					"bit index may not exceed pin bit capacity:\n{bit_index} >= {size}\n(in '{ident}')"
				);
			}
		};

		Ok(Self {
			pin,
			line: bit_index.unwrap_or_default(),
		})
	}
}
