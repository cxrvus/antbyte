use crate::ant::pin::{Pin, PinValue};

use super::{Behavior, World};

fn zero_count_mask(x: u8) -> u8 {
	0xff_u8.unbounded_shr(8 - x.trailing_zeros())
}

impl World {
	pub(super) fn ant_tick(&mut self, ant_index: usize) {
		let ant = self.ants[ant_index];

		let Behavior {
			inputs,
			outputs,
			logic: truth_table,
			..
		} = self
			.get_behavior(ant.behavior)
			.cloned()
			.expect("invalid Behavior ID");

		let mut input_bits = 0u8;

		use Pin::*;

		for input_sub_pin in inputs.iter() {
			let input_value: u8 = match input_sub_pin.pin {
				Time => ant.age as u8,
				Pulse => zero_count_mask(ant.age as u8),
				Cell => self.cells.at(&ant.pos.sign()).unwrap().value,
				CellNext => self
					.next_pos(&ant)
					.map(|pos| self.cells.at(&pos.sign()).unwrap().value)
					.unwrap_or(0u8),
				Memory => ant.memory,
				Random => self.rng(),
				Chance => zero_count_mask(self.rng()),
				Obstacle => match self.next_pos(&ant) {
					Some(pos) => self.is_occupied(&pos).into(),
					None => 1,
				},
				_ => panic!("unhandled input: {input_sub_pin:?}"),
			};

			let bit_index = input_sub_pin.line;
			let masked_input_value = (input_value >> bit_index) & 1;
			input_bits <<= 1;
			input_bits |= masked_input_value;
		}

		// calculating the output
		let mut output_bits = truth_table.get(input_bits);

		// condense output bits into bytes
		let mut output_values: Vec<PinValue> = vec![];

		let mut output_cell_mask = 0u8;

		for output_sub_pin in outputs.iter().rev() {
			let output_bit = (output_bits & 1) as u8;
			let bit_index = output_sub_pin.line;
			let new_value = output_bit << bit_index;

			// only overwrite targeted cell bits
			if let Pin::Cell = output_sub_pin.pin {
				output_cell_mask |= 1 << output_sub_pin.line;
			}

			if let Some(output_value) = output_values
				.iter_mut()
				.find(|output_value| output_value.pin == output_sub_pin.pin)
			{
				output_value.value |= new_value;
			} else {
				output_values.push(PinValue {
					pin: output_sub_pin.pin,
					value: new_value,
				});
			}

			output_bits >>= 1;
		}

		output_values.sort();

		let mut ant = ant;

		// invert mask to only keep bits that are not targeted
		output_cell_mask = !output_cell_mask;

		let mut halted = false;

		for PinValue { pin: output, value } in output_values.into_iter() {
			match (output, value) {
				(Direction, _) => ant.set_dir(ant.dir + value),
				(Halted, _) => halted = value != 0,
				(Cell, _) => {
					let old_value = self.cells.at(&ant.pos.sign()).unwrap().value;
					let value = value | (old_value & output_cell_mask);
					let adjusted = self.adjusted_color(value);
					self.set_value(&ant.pos, adjusted);
				}
				(CellClear, 1) => self.set_value(&ant.pos, 0),
				(Memory, value) => ant.memory = value,
				(SpawnAnt, _) if value != 0 => self.reproduce(&ant, value),
				(Kill, 1) => {
					if let Some(pos) = self.next_pos(&ant) {
						self.kill_at(&pos);
					}
				}
				(Die, 1) => self.die(&mut ant),
				_ => {}
			};
		}

		if ant.is_alive() && !halted {
			self.move_tick(&mut ant);
		}

		ant.age += 1;

		self.ants[ant_index] = ant;
	}
}
