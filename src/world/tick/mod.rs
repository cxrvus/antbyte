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

		// tick ants (sync)
		let image = self.state.clone();

		let all_outputs: Vec<_> = image
			.ants
			.iter()
			.map(|(pos, ant)| (pos, self.get_output(ant, *pos)))
			.collect();

		for (pos, outputs) in all_outputs {
			self.sync_tick(*pos, &outputs);
		}

		// tick ants (async)
		self.kill_tick();
		self.move_tick();
		self.spawn_tick();
		self.die_tick();

		// cell decay
		if self.config().decay.is_some() {
			self.cell_decay();
		}

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
}
