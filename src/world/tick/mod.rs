use crate::world::World;
mod tick_async;
mod tick_sync;
mod tick_util;

impl World {
	pub(super) fn tick(&mut self) -> bool {
		// end world if conditions are met
		let no_ants = self.ants.is_empty();

		let tick_overflow = self
			.config()
			.max_ticks
			.map(|max| self.tick_count >= max)
			.unwrap_or_default();

		if no_ants || tick_overflow {
			return false;
		}

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

		// todo: optimize decay
		// cell decay
		if self.config().decay.is_some() {
			self.cell_decay();
		}

		self.tick_count += 1;

		true
	}
}
