use std::fmt::Display;

use anyhow::{Result, anyhow};

#[cfg_attr(test, derive(ts_rs::TS))]
#[derive(Debug, Clone, Default)]
pub struct TruthTable {
	input_count: u8,
	output_count: u8,
	entries: Vec<u32>,
}

impl TruthTable {
	pub fn new(input_bits: usize, output_bits: usize, entries: Vec<u32>) -> Result<Self> {
		if output_bits > 32 {
			Err(anyhow!("output bit count must not be greater than 32"))
		} else if entries.len() != 1 << input_bits {
			Err(anyhow!("entry count must be equal to [1 << input_bits]"))
		} else if let Some(index) = entries
			.iter()
			.position(|x| output_bits < 32 && *x >= 1 << output_bits)
		{
			Err(anyhow!(
				"all entries must be less than {} (1 << output_bits)\nfound {} at index {}",
				1u64 << output_bits,
				entries[index],
				index
			))
		} else {
			let packed_len = (entries.len() * output_bits).div_ceil(32);
			let mut packed = vec![0; packed_len];

			if output_bits > 0 {
				for (index, entry) in entries.into_iter().enumerate() {
					let bit_offset = index * output_bits;
					let word_index = bit_offset / 32;
					let bit_index = bit_offset % 32;
					let value = (entry as u64) << bit_index;

					packed[word_index] |= value as u32;
					if bit_index + output_bits > 32 {
						packed[word_index + 1] |= (value >> 32) as u32;
					}
				}
			}

			Ok(Self {
				input_count: input_bits as u8,
				output_count: output_bits as u8,
				entries: packed,
			})
		}
	}

	pub fn input_count(&self) -> u8 {
		self.input_count
	}

	pub fn output_count(&self) -> u8 {
		self.output_count
	}

	pub fn entries(&self) -> Vec<u32> {
		(0..(1usize << self.input_count))
			.map(|input| self.get(input))
			.collect()
	}

	pub fn get(&self, input: usize) -> u32 {
		if input >= 1 << self.input_count {
			return 0;
		}

		let bit_offset = input * self.output_count as usize;
		let word_index = bit_offset / 32;
		let bit_index = bit_offset % 32;
		let value = (self.entries.get(word_index).copied().unwrap_or_default() as u64)
			| ((self
				.entries
				.get(word_index + 1)
				.copied()
				.unwrap_or_default() as u64)
				<< 32);

		(value >> bit_index) as u32 & self.output_mask()
	}

	fn output_mask(&self) -> u32 {
		if self.output_count == 32 {
			u32::MAX
		} else {
			(1 << self.output_count) - 1
		}
	}
}

impl Display for TruthTable {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for (input, output) in self.entries().iter().enumerate() {
			writeln!(f, "{input:08b} => {output:08b}")?;
		}

		writeln!(f)
	}
}

#[cfg(test)]
mod tests {
	use super::TruthTable;

	#[test]
	fn packs_values_across_words() {
		let entries = (0..16).collect::<Vec<_>>();
		let table = TruthTable::new(4, 4, entries.clone()).unwrap();

		assert_eq!(table.entries(), entries);
		assert_eq!(table.get(16), 0);
	}

	#[test]
	fn supports_zero_and_32_bit_outputs() {
		let zeroes = TruthTable::new(1, 0, vec![0, 0]).unwrap();
		assert_eq!(zeroes.entries(), vec![0, 0]);

		let values = vec![u32::MAX, 0];
		let table = TruthTable::new(1, 32, values.clone()).unwrap();
		assert_eq!(table.entries(), values);
	}

	#[test]
	fn rejects_values_outside_output_width() {
		assert!(TruthTable::new(1, 3, vec![0, 8]).is_err());
	}
}
