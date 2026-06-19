use std::{
	collections::{BTreeMap, BTreeSet},
	mem::swap,
};

use crate::{
	ant::Ant,
	util::vec2::Position,
	world::{World, config::BorderMode, state::Ants},
};

enum MoveAction {
	Stay,
	Move(Position),
	Nop,
}

impl World {
	pub(super) fn kill_tick(&mut self, layer: u8) {
		let mut kills = BTreeSet::new();

		for (pos, ant) in &self.ants[&layer].clone() {
			if ant.will_kill
				&& !ant.waiting()
				&& let Some(next_pos) = self.next_pos(*pos, layer, ant.dir)
				&& self.ants[&layer].contains_key(&next_pos)
			{
				kills.insert(next_pos);
			}
		}

		self.ants
			.layer_mut(layer)
			.retain(|pos, _| !kills.contains(pos));
	}

	pub(super) fn end_tick(&mut self, layer: u8) {
		// die
		self.ants.layer_mut(layer).retain(|_, ant| !ant.will_die);

		// wait
		for ant in &mut self.ants.layer_mut(layer).values_mut() {
			if ant.will_wait {
				ant.will_wait = false;
			} else if ant.waiting() {
				ant.wait_ticks -= 1;
			}
		}
	}

	pub(super) fn move_tick(&mut self, layer: u8) {
		let mut source = Ants::new();
		let mut result = Ants::new();

		swap(self.ants.layer_mut(layer), &mut source);

		while let Some((pos, ant)) = source.pop_first() {
			let mut stack = vec![(pos, ant)];

			// used to resolve cycles
			let mut cycle_pos: Option<Position> = None;

			while let Some((pos, ant)) = stack.pop() {
				let action = if ant.halted() {
					MoveAction::Stay
				} else if let Some(cycle_pos_value) = cycle_pos {
					if pos == cycle_pos_value {
						// reached last ant in cycle
						cycle_pos = None;
					}

					let target_pos = self
						.next_pos(pos, layer, ant.dir)
						.expect("no target position for ant in cycle");

					// all ants in cycle can move
					MoveAction::Move(target_pos)
				} else if let Some(target_pos) = self.next_pos(pos, layer, ant.dir) {
					if result.contains_key(&target_pos) {
						// target pos is occupied in result => can't move
						MoveAction::Stay
					} else if let Some(&target_ant) = source.get(&target_pos) {
						// target pos is occupied in source
						if target_ant.halted() {
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
								.get_contestants(&source, target_pos, layer)
								.iter()
								.map(|pos| source[pos])
								.collect::<Vec<_>>();

							if contestants.is_empty() || self.luck_check(layer, &contestants, &ant)
							{
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
					match self.border_mode(layer) {
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

		fn commit(result: &mut Ants, pos: Position, ant: Ant) {
			let prev = result.insert(pos, ant);
			assert!(prev.is_none(), "tried to occupy occupied space")
		}

		swap(&mut result, self.ants.layer_mut(layer));
	}

	pub(super) fn spawn_tick(&mut self, source_layer: u8) {
		let mut claims = BTreeMap::<(Position, u8), Vec<Position>>::new();

		if self.ants.ant_count() >= self.config().ant_limit as usize {
			return;
		}

		for (pos, ant) in &self.ants[&source_layer] {
			if let Some(target_pos) = self.next_pos(*pos, source_layer, ant.dir.inverted())
				&& ant.child_behavior != 0
				&& !ant.waiting()
				&& self.get_behavior(ant.child_behavior).is_some()
			{
				let target_layer = source_layer + ant.child_layer;

				let target_layer_in_bounds = target_layer < self.config().layers;

				let target_pos_occupied = self
					.ants
					.get(&target_layer)
					.is_some_and(|ants| ants.contains_key(&target_pos));

				if target_layer_in_bounds && !target_pos_occupied {
					claims
						.entry((target_pos, target_layer))
						.or_default()
						.push(*pos);
				}
			}
		}

		let mut new_ants: Vec<(Position, u8, Ant)> = vec![];

		// resolve target position conflicts
		for ((target_pos, target_layer), contestant_positions) in claims {
			let contestants = contestant_positions
				.iter()
				.map(|pos| self.ants[&source_layer][pos])
				.collect::<Vec<_>>();

			// conflict resolution
			let ant = contestants
				.iter()
				.find(|ant| self.luck_check(target_layer, &contestants, ant))
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

			new_ants.push((target_pos, target_layer, new_ant));
		}

		for (pos, layer, ants) in new_ants.into_iter() {
			self.ants.entry(layer).or_default().insert(pos, ants);
		}
	}
}
