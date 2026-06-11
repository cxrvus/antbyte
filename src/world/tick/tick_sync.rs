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
				Clear => (*self.cells.get(pos).unwrap() == 0) as u8,
				NearbyCell => next_pos
					.map(|pos| *self.cells.get(pos).unwrap())
					.unwrap_or(0u8),

				Init => (ant.birth_tick + 1 == self.tick_count()) as u8,
				Time => ant.clock,
				Pulse => zero_count_mask(ant.clock),
				Mem => ant.memory,
				Random => self.rng(),
				Chance => zero_count_mask(self.rng()),

				NearbyAnt => {
					(next_ant.is_some()
						|| (self.config().border == BorderMode::Collide && next_pos.is_none()))
						as u8
				}

				NearbyId => next_ant.map(|next| next.behavior).unwrap_or_default(),
				NearbyMem => next_ant.map(|next| next.memory).unwrap_or_default(),

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

	pub(super) fn get_output(&self, ant: &Ant, input: u8) -> Vec<PinValue> {
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

	pub(super) fn sync_tick(&mut self, pos: Position, input: u8, output: &[PinValue]) {
		let mut ant = self.ants[&pos];

		let behavior = self
			.get_behavior(ant.behavior)
			.cloned()
			.expect("invalid Behavior ID");

		let cell_mask = behavior.pin_mask(Pin::Cell);
		let mem_mask = behavior.pin_mask(Pin::Mem);

		let mut clear = false;

		for pin_value in output.iter() {
			let PinValue { pin, value } = *pin_value;
			let value_bool = value != 0;

			match (pin, value_bool) {
				(Clear, true) => clear = true,
				(Cell, _) => self.set_cell(pos, value, cell_mask),

				(SpawnDir, _) => ant.child_dir = Direction::from(value),
				(SpawnMem, _) => ant.child_memory = value,

				(Mem, _) => ant.memory = value | (ant.memory & !mem_mask),

				(Wait, true) => {
					ant.will_wait = true;
					ant.wait_ticks = value
				}

				(Signal, true) => self.signal_out |= value,
				(ExtOut, true) => self.ext_output.push(value),

				// deferred to async ticks...

				// kill_tick
				(Kill, _) => ant.will_kill = value_bool,

				// move_tick
				(Halt, _) => ant.will_halt = value_bool,
				(Dash, _) => ant.will_dash = value_bool,

				(Dir, true) => ant.dir += Direction::from(value),

				// spawn_tick
				(SpawnId, _) => ant.child_behavior = value,

				// die_tick
				(Die, _) => ant.will_die = value_bool,
				_ => {}
			};
		}

		ant.last_input = input;
		ant.clock = ant.clock.wrapping_add(1);

		if clear {
			self.set_cell(pos, 0, !cell_mask);
		}

		self.ants.insert(pos, ant);
	}
}
