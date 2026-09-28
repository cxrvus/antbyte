use crate::{ant::Ant, util::vec2::Pos, world::World};
use std::collections::{BTreeMap, BTreeSet};

impl World {
	pub(super) fn kill_tick(&mut self, layer: u8) {
		let mut kills = BTreeSet::new();

		for (pos, ant) in &self.ants[&layer].clone() {
			if ant.will_kill
				&& !ant.waiting()
				&& let Some(next_pos) = self.next_pos(*pos, layer, ant.look_dir())
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

	pub(super) fn spawn_tick(&mut self, source_layer: u8) {
		let mut claims = BTreeMap::<(Pos, u8), Vec<Pos>>::new();

		if self.ants.ant_count() >= self.config().ant_limit as usize {
			return;
		}

		for (pos, ant) in &self.ants[&source_layer] {
			if let Some(target_pos) = self.next_pos(*pos, source_layer, ant.look_dir().flipped())
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

		let mut new_ants: Vec<(Pos, u8, Ant)> = vec![];

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
			let rotation = ant.child_rotation.rotated(ant.rotation.value(), false);

			let new_ant = Ant {
				rotation,
				birth_rot: rotation,
				behavior: ant.child_behavior,
				memory: ant.child_memory,
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
