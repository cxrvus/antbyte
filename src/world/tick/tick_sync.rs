use crate::{
	ant::{
		Ant,
		behavior::Behavior,
		pin::{Pin, PinValue},
	},
	util::{dir::Direction, vec2::Position},
	world::World,
};

fn zero_count_mask(x: u8) -> u8 {
	0xff_u8.unbounded_shr(8 - x.trailing_zeros())
}

use Pin::*;
impl World {
	pub(super) fn get_output(&mut self, ant: &Ant, pos: Position) -> Vec<PinValue> {
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
				Cell => self.cells.at(pos).unwrap().value,
				Next => self
					.next_pos(pos, ant.dir)
					.map(|pos| self.cells.at(pos).unwrap().value)
					.unwrap_or(0u8),
				Mem => ant.memory,
				Random => self.rng(),
				Chance => zero_count_mask(self.rng()),
				Collide => match self.next_pos(pos, ant.dir) {
					Some(pos) => self.ants.contains_key(&pos).into(),
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

		for output_sub_pin in outputs.iter().rev() {
			let output_bit = (output_bits & 1) as u8;
			let bit_index = output_sub_pin.line;
			let new_value = output_bit << bit_index;

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

		output_values
	}

	pub(super) fn sync_tick(&mut self, pos: Position, outputs: &Vec<PinValue>) {
		let mut ant = self.ants[&pos];

		let cell_mask = self
			.get_behavior(ant.behavior)
			.cloned()
			.expect("invalid Behavior ID")
			.cell_mask();

		let mut clear = false;

		for PinValue { pin, value } in outputs {
			match (pin, value) {
				(Clear, 1) => clear = true,
				(Cell, _) => self.set_cell(pos, *value, cell_mask),

				(AntDir, value) => ant.child_dir = Direction::new(*value),
				(AntMem, value) => ant.child_memory = *value,
				(Mem, value) => ant.memory = *value,

				(Send, value) => self.event_out |= value,
				(ExtOut, value) => self.ext_output.push(*value),

				// deferred to async ticks...

				// kill_tick
				(Kill, value) => ant.kill = *value != 0,

				// move_tick
				(Halt, _) => ant.halt = *value != 0,
				(Dir, _) => ant.dir += Direction::new(*value),

				// spawn_tick
				(AntSpawn, _) => ant.child_behavior = *value,

				// die_tick
				(Die, 1) => ant.die = true,
				_ => {}
			};
		}

		if clear {
			self.set_cell(pos, 0, !cell_mask);
		}

		self.ants.insert(pos, ant);
	}
}
