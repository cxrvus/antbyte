use crate::{
	ant::{
		Ant,
		pin::{Pin, PinValue},
	},
	util::{dir::Direction, vec2::Position},
	world::{World, config::BorderMode},
};

fn zero_count_mask(x: u8) -> u8 {
	0xff_u8.unbounded_shr(8 - x.trailing_zeros())
}

use Pin::*;
impl World {
	pub(super) fn get_input(&mut self, ant: &Ant, pos: Position) -> u8 {
		let behavior = self
			.get_behavior(ant.behavior)
			.cloned()
			.expect("invalid Behavior ID");

		let mut input_bits = 0u8;

		for input_sub_pin in behavior.inputs.iter() {
			let next_pos = self.next_pos(pos, ant.dir);
			let next_ant = next_pos.and_then(|pos| self.ants.get(&pos));

			let input_value: u8 = match input_sub_pin.pin {
				Cell => *self.cells.get(pos).unwrap(),
				NextCell => next_pos
					.map(|pos| *self.cells.get(pos).unwrap())
					.unwrap_or(0u8),

				Init => (ant.birth_tick + 1 == self.tick_count()) as u8,
				Time => ant.age(self.tick_count()) as u8,
				Pulse => zero_count_mask(ant.age(self.tick_count()) as u8),
				Mem => ant.memory,
				Random => self.rng(),
				Chance => zero_count_mask(self.rng()),

				NextObstacle => {
					(next_ant.is_some()
						|| (self.config().border_mode == BorderMode::Collide && next_pos.is_none()))
						as u8
				}

				NextId => next_ant.map(|next| next.behavior).unwrap_or_default(),
				NextMem => next_ant.map(|next| next.memory).unwrap_or_default(),

				Signal => self.signal_in,
				ExtIn => self.ext_input,
				_ => panic!("unhandled input: {input_sub_pin:?}"),
			};

			let bit_index = input_sub_pin.line;
			let masked_input_value = (input_value >> bit_index) & 1;
			input_bits <<= 1;
			input_bits |= masked_input_value;
		}

		input_bits
	}

	pub(super) fn get_output(&mut self, ant: &Ant, input: u8) -> Vec<PinValue> {
		let behavior = self
			.get_behavior(ant.behavior)
			.cloned()
			.expect("invalid Behavior ID");

		// calculating the output
		let mut output_bits = behavior.logic.get(input);

		// condense output bits into bytes
		let mut output_values: Vec<PinValue> = vec![];

		for output_sub_pin in behavior.outputs.iter().rev() {
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

	pub(super) fn sync_tick(&mut self, pos: Position, input: u8, output: &Vec<PinValue>) {
		let mut ant = self.ants[&pos];

		let behavior = self
			.get_behavior(ant.behavior)
			.cloned()
			.expect("invalid Behavior ID");

		let cell_mask = behavior.pin_mask(Pin::Cell);
		let mem_mask = behavior.pin_mask(Pin::Mem);

		let mut clear = false;

		for PinValue { pin, value } in output {
			match (pin, value) {
				(Clear, 1) => clear = true,
				(Cell, _) => self.set_cell(pos, *value, cell_mask),

				(SpawnDir, value) => ant.child_dir = Direction::from(*value),
				(SpawnMem, value) => ant.child_memory = *value,
				(Mem, value) => ant.memory = *value | (ant.memory & !mem_mask),

				(Signal, value) => self.signal_out |= value,
				(ExtOut, value) => self.ext_output.push(*value),

				// deferred to async ticks...

				// kill_tick
				(Kill, value) => ant.kill = *value != 0,

				// move_tick
				(Halt, _) => ant.halt = *value != 0,
				(Dash, _) => ant.dash = *value != 0,
				(Dir, _) => ant.dir += Direction::from(*value),

				// spawn_tick
				(SpawnId, _) => ant.child_behavior = *value,

				// die_tick
				(Die, 1) => ant.die = true,
				_ => {}
			};
		}

		ant.last_input = input;

		if clear {
			self.set_cell(pos, 0, !cell_mask);
		}

		self.ants.insert(pos, ant);
	}
}
