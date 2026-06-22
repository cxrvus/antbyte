use crate::world::World;
mod tick_async;
mod tick_sync;
mod tick_util;

impl World {
	/// advances simulation by 1 tick and returns false if this is supposed to be the last tick
	pub(super) fn tick(&mut self) -> bool {
		if self.tick_count == u32::MAX {
			return false;
		}

		self.tick_count += 1;

		// signals
		self.signal_in = self.signal_out;
		self.signal_out = 0;

		// tile decay
		if self.config().decay.is_some() {
			self.tile_decay();
		}

		// ants
		for layer in (0..self.config().layers).rev() {
			if self.ants.get(&layer).is_some() {
				self.tick_layer(layer);
			}
		}

		self.ants.retain(|_, ants| !ants.is_empty());

		// end world if conditions are met
		let no_ants = self.ants.is_empty();

		let tick_overflow = self.tick_count == u32::MAX;

		let max_tick = self
			.config()
			.max_ticks
			.map(|max| self.tick_count >= max)
			.unwrap_or_default();

		!(no_ants || tick_overflow || max_tick)
	}

	fn tick_layer(&mut self, layer: u8) {
		// tick ants (sync)
		let mut all_outputs = vec![];

		for (pos, ant) in self.ants[&layer].clone() {
			if !ant.waiting() {
				let input = self.get_input(&ant, pos, layer);
				let output = self.get_output(&ant, input);
				all_outputs.push((pos, input, output));
			}
		}

		for (pos, input, output) in all_outputs {
			self.sync_tick(pos, layer, input, &output);
		}

		// tick ants (async)
		self.kill_tick(layer);
		self.move_tick(layer);
		self.spawn_tick(layer);
		self.end_tick(layer);
	}
}
