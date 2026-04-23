use std::collections::{BTreeMap, BTreeSet};

use crate::{
	ant::{
		Ant,
		pin::{Pin, PinValue},
	},
	util::{dir::Direction, vec2::Vec2u},
	world::config::BorderMode,
};

use super::{Behavior, World};

use Pin::*;

fn zero_count_mask(x: u8) -> u8 {
	0xff_u8.unbounded_shr(8 - x.trailing_zeros())
}

enum MoveAction {
	Stay,
	Move(Vec2u),
	Nop,
}

impl World {
	pub(super) fn get_output(&mut self, ant: &Ant, pos: Vec2u) -> Vec<PinValue> {
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

	pub(super) fn sync_tick(&mut self, pos: Vec2u, outputs: &Vec<PinValue>) {
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

	pub(super) fn kill_tick(&mut self) {
		let mut kills = BTreeSet::new();

		for (pos, ant) in &self.ants.clone() {
			if ant.kill
				&& let Some(next_pos) = self.next_pos(*pos, ant.dir)
				&& self.ants.contains_key(&next_pos)
			{
				kills.insert(next_pos);
			}
		}

		self.ants.retain(|pos, _| !kills.contains(pos));
	}

	pub(super) fn die_tick(&mut self) {
		self.ants.retain(|_, ant| !ant.die);
	}

	pub(super) fn move_tick(&mut self) {
		let mut source = self.ants.clone();
		let mut result = BTreeMap::new();

		while let Some((pos, ant)) = source.pop_first() {
			let mut stack = vec![(pos, ant)];

			// used to resolve cycled
			let mut cycle_pos: Option<Vec2u> = None;

			while let Some((pos, ant)) = stack.pop() {
				let action = if ant.halt {
					MoveAction::Stay
				} else if let Some(cycle_pos_value) = cycle_pos {
					if pos == cycle_pos_value {
						// reached last ant in cycle
						cycle_pos = None;
					}

					let target_pos = self
						.next_pos(pos, ant.dir)
						.expect("no target position for ant in cycle");

					// all ants in cycle can move
					MoveAction::Move(target_pos)
				} else if let Some(target_pos) = self.next_pos(pos, ant.dir) {
					if result.contains_key(&target_pos) {
						// target pos is occupied in result => can't move
						MoveAction::Stay
					} else if let Some(&target_ant) = source.get(&target_pos) {
						// target pos is occupied in source
						if target_ant.halt {
							// dead end => stay
							MoveAction::Stay
						} else {
							// chain => recurse
							stack.push((pos, ant));
							source.remove(&target_pos);
							stack.push((target_pos, target_ant));
							MoveAction::Nop
						}
					} else {
						// target pos is free in source

						if stack.iter().any(|(visited, _)| target_pos == *visited) {
							// target is already part of the chain
							// cycle => resolve
							cycle_pos = Some(target_pos);
							MoveAction::Move(target_pos)
						} else {
							let contestants = self
								.get_contestants(&source, target_pos)
								.iter()
								.map(|pos| source[pos])
								.collect::<Vec<_>>();

							if contestants.is_empty() || self.luck_check(&contestants, &ant) {
								// target is uncontested or conflict has been won => move
								MoveAction::Move(target_pos)
							} else {
								// conflict has been lost => stay
								MoveAction::Stay
							}
						}
					}
				} else {
					// target pos is outside of grid
					match self.config().border_mode {
						BorderMode::Collide => MoveAction::Stay,
						BorderMode::Despawn => MoveAction::Nop,
						_ => panic!("no target position, despite border mode guaranteeing one"),
					}
				};

				match action {
					MoveAction::Stay => commit(&mut result, pos, ant),
					MoveAction::Move(target_pos) => commit(&mut result, target_pos, ant),
					MoveAction::Nop => { /* ant will not be committed to result */ }
				}
			}

			// reached end of ant chain
		}

		fn commit(result: &mut BTreeMap<Vec2u, Ant>, pos: Vec2u, ant: Ant) {
			let prev = result.insert(pos, ant);
			assert!(prev.is_none(), "tried to occupy occupied space")
		}

		self.ants = result;
	}

	const ANT_LIMIT: u32 = 0x100;

	pub(super) fn spawn_tick(&mut self) {
		let mut claims = BTreeMap::<Vec2u, Vec<Vec2u>>::new();

		let ant_limit = self.config().ant_limit.unwrap_or(Self::ANT_LIMIT) as usize;

		if self.ants.len() >= ant_limit {
			return;
		}

		for (pos, ant) in &self.ants {
			if ant.child_behavior == 0 {
				continue;
			} else if let Some(target_pos) = self.next_pos(*pos, ant.dir.inverted())
				&& self.get_behavior(ant.child_behavior).is_some()
			{
				claims.entry(target_pos).or_default().push(*pos);
			}
		}

		let mut new_ants = BTreeMap::new();

		// resolve target position conflicts
		for (target_pos, contestant_positions) in claims {
			let contestants = contestant_positions
				.iter()
				.map(|pos| self.ants[pos])
				.collect::<Vec<_>>();

			// conflict resolution
			let ant = contestants
				.iter()
				.find(|ant| self.luck_check(&contestants, ant))
				.unwrap();

			// spawn
			let child_dir = ant.dir + ant.child_dir;

			let new_ant = Ant {
				behavior: ant.child_behavior,
				memory: ant.child_memory,
				dir: child_dir,
				birth_tick: self.tick_count,
				..Default::default()
			};

			new_ants.insert(target_pos, new_ant);
		}

		self.ants.extend(new_ants);
	}
}
