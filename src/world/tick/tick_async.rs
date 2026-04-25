use std::collections::{BTreeMap, BTreeSet};

use crate::{
	ant::Ant,
	util::vec2::Position,
	world::{Ants, World, config::BorderMode},
};

enum MoveAction {
	Stay,
	Move(Position),
	Nop,
}

impl World {
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
		let mut result = Ants::new();

		while let Some((pos, ant)) = source.pop_first() {
			let mut stack = vec![(pos, ant)];

			// used to resolve cycles
			let mut cycle_pos: Option<Position> = None;

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

		fn commit(result: &mut Ants, pos: Position, ant: Ant) {
			let prev = result.insert(pos, ant);
			assert!(prev.is_none(), "tried to occupy occupied space")
		}

		self.ants = result;
	}

	const ANT_LIMIT: u32 = 0x100;

	pub(super) fn spawn_tick(&mut self) {
		let mut claims = BTreeMap::<Position, Vec<Position>>::new();

		let ant_limit = self.config().ant_limit.unwrap_or(Self::ANT_LIMIT) as usize;

		if self.ants.len() >= ant_limit {
			return;
		}

		for (pos, ant) in &self.ants {
			if ant.child_behavior == 0 {
				continue;
			} else if let Some(target_pos) = self.next_pos(*pos, ant.dir.inverted())
				&& !self.ants.contains_key(&target_pos)
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
