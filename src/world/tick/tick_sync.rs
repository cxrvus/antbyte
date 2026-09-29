use crate::{
	ant::{
		Ant,
		pin::{Pin, PinValue},
	},
	util::{dir::Direction, vec2::Pos},
	world::{World, config::BorderMode},
};

fn zero_count_mask(x: u8) -> u8 {
	0xff_u8.unbounded_shr(8 - x.trailing_zeros())
}

use Pin::*;
impl World {
	pub(super) fn get_input(&mut self, ant: &Ant, pos: Pos, layer: u8) -> u8 {
		let behavior = self
			.get_behavior(ant.behavior)
			.cloned()
			.expect("invalid Behavior ID");

		let mut input_bits = 0u8;

		for input_sub_pin in behavior.inputs.iter() {
			let target_dir = Direction::from_u8(input_sub_pin.channel()) + ant.look_dir();
			let target_pos = self.next_pos(pos, layer, target_dir);
			let target_ant = target_pos.and_then(|pos| self.ants[&layer].get(&pos));

			let input_value: u8 = match input_sub_pin.pin {
				TileColor => *self.tiles.get(pos).unwrap(),
				TileZero => (*self.tiles.get(pos).unwrap() == 0) as u8,
				NearbyTile => target_pos
					.map(|pos| *self.tiles.get(pos).unwrap())
					.unwrap_or(0u8),

				BirthTick => (ant.birth_tick + 1 == self.tick_count()) as u8,
				Halt => ant.halted() as u8,

				Counter => ant.counter,
				Clock => zero_count_mask(ant.counter),
				Noise => self.rng(),
				Probability => zero_count_mask(self.rng()),

				AntId => ant.behavior,

				Rotation => ant.rotation.rotated(ant.birth_rot.value(), true).value(),
				RotateZero => (ant.rotation == ant.birth_rot) as u8,

				NearbyAnt => {
					(target_ant.is_some()
						|| (self.border_mode(layer) == BorderMode::Collide && target_pos.is_none()))
						as u8
				}

				NearbyId => target_ant.map(|target| target.behavior).unwrap_or_default(),
				NearbyMem => target_ant.map(|target| target.memory).unwrap_or_default(),

				Mem => ant.memory,
				Signal => self.signal_in,

				// need channel because ext_input is 16 bits
				ExtIn => (self.ext_input >> (input_sub_pin.channel() * 8)) as u8,

				_ => panic!("unhandled input: {input_sub_pin:?}"),
			};

			let masked_input_value = (input_value >> input_sub_pin.line()) & 1;
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
			let output_bit = (output_bits & 1) as u16;
			let new_value = output_bit << output_sub_pin.bit_index;

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

	pub(super) fn sync_tick(&mut self, pos: Pos, layer: u8, input: u8, output: &[PinValue]) {
		let mut ant = self.ants[&layer][&pos];

		let behavior = self
			.get_behavior(ant.behavior)
			.cloned()
			.expect("invalid Behavior ID");

		let tile_mask = behavior.pin_mask(Pin::TileColor);
		let mem_mask = behavior.pin_mask(Pin::Mem);

		let mut clear = false;
		let (mut rot, mut left, mut rot_zero) = (0u8, false, false);
		let (mut child_rot, mut child_left) = (0u8, false);

		ant.child_rotation.reset();

		for pin_value in output.iter() {
			let PinValue { pin, value } = *pin_value;
			let value_bool = value != 0;
			let wide_value = value;
			let value = value as u8;

			match (pin, value_bool) {
				(Mem, _) => ant.memory = value | (ant.memory & !mem_mask),
				(Signal, true) => self.signal_out |= value,
				(ExtOut, true) => self.ext_output.push(wide_value),

				// tiles
				(TileZero, true) => clear = true,
				(TileColor, _) => self.set_tile(pos, value, tile_mask),

				// deferred to async ticks...

				// kill_tick
				(Kill, _) => ant.will_kill = value_bool,

				// move_tick
				(Halt, _) => ant.will_halt = value_bool,
				(TieBreaker, _) => ant.tie_breaker = value_bool,

				(Rotation, true) => rot = value.reverse_bits(),
				(RotateLeft, true) => left = true,
				(RotateZero, true) => rot_zero = true,

				// spawn_tick
				(AntId, _) => ant.child_behavior = value,
				(ChildLayer, _) => ant.child_layer = value,
				(ChildRotation, _) => child_rot = value.reverse_bits(),
				(ChildLeft, true) => child_left = true,
				(ChildMem, _) => ant.child_memory = value,

				// end_tick
				(Die, _) => ant.will_die = value_bool,

				(Wait, true) => {
					ant.will_wait = true;
					ant.wait_ticks = value
				}

				// ignored
				_ => {}
			};
		}

		ant.last_input = input;
		ant.counter = ant.counter.wrapping_add(1);

		if clear {
			self.set_tile(pos, 0, !tile_mask);
		}
		if rot_zero {
			ant.rotation = ant.birth_rot;
		}

		let inv_rot = self.config().inv_rot;
		(left, child_left) = (left ^ inv_rot, child_left ^ inv_rot);

		ant.rotation.rotate(rot, left);
		ant.child_rotation.rotate(child_rot, child_left);

		ant.update_move_dir();

		self.ants.get_mut(&layer).unwrap().insert(pos, ant);
	}
}
