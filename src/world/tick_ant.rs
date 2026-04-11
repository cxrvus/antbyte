use crate::ant::{
	Ant,
	pin::{Pin, PinValue},
};

use super::{Behavior, World};

use Pin::*;

fn zero_count_mask(x: u8) -> u8 {
	0xff_u8.unbounded_shr(8 - x.trailing_zeros())
}

impl World {
	pub(super) fn get_output(&mut self, ant: &Ant) -> Vec<PinValue> {
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

		for input_sub_pin in inputs.iter() {
			let input_value: u8 = match input_sub_pin.pin {
				Time => ant.age(self.tick_count) as u8,
				Pulse => zero_count_mask(ant.age(self.tick_count) as u8),
				Cell => self.cells.at(&ant.pos.sign()).unwrap().value,
				Next => self
					.next_pos(ant)
					.map(|pos| self.cells.at(&pos.sign()).unwrap().value)
					.unwrap_or(0u8),
				Mem => ant.memory,
				Random => self.rng(),
				Chance => zero_count_mask(self.rng()),
				Collide => match self.next_pos(ant) {
					Some(pos) => self.is_occupied(&pos).into(),
					None => 1,
				},
				Event => self.event_in,
				ExtIn => self.ext_input,
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
					mask: output_cell_mask,
				});
			}

			output_bits >>= 1;
		}

		output_values
	}

	pub(super) fn sync_tick(&mut self, ant_index: usize, outputs: &Vec<PinValue>) {
		let mut ant = self.ants[ant_index];

		let mut cell_mask = 0;
		let mut clear = false;

		let mut halted = false;

		let mut child_dir = 0;
		let mut child_mem = 0;

		for PinValue { pin, value, mask } in outputs {
			match (pin, value) {
				(Cell, _) => {
					cell_mask = *mask;
					self.set_cell(&ant, *value, cell_mask);
				}
				(Clear, 1) => clear = true,

				(Mem, value) => ant.memory = *value,
				(Send, value) => self.event_out |= value,
				(ExtOut, value) => self.ext_output.push(*value),

				// TODO: kill tick
				(Kill, 1) => {
					if let Some(pos) = self.next_pos(&ant) {
						self.kill_at(&pos);
					}
				}

				// TODO: move tick
				(Dir, _) => ant.set_dir(ant.dir + value),
				(Halt, _) => halted = *value != 0,

				// TODO: spawn tick
				(AntSpawn, _) if *value != 0 => self.reproduce(&ant, *value, child_dir, child_mem),
				(AntDir, value) => child_dir = *value,
				(AntMem, value) => child_mem = *value,

				// TODO: die tick
				(Die, 1) => self.die(&mut ant),
				_ => {}
			};
		}

		if clear {
			self.set_cell(&ant, 0, !cell_mask);
		}

		if ant.is_alive() && !halted && !ant.is_queen() {
			self.move_ant(&mut ant);
		}

		self.ants[ant_index] = ant;
	}
}
