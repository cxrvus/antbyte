use crate::{
	ui::term::render::{clear_screen, print_title_short},
	world::World,
};
mod tick_async;
mod tick_sync;
mod tick_util;

const MAX_TICKS: u32 = 1 << 16;

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

	pub(super) fn tick_all(&mut self) {
		let max_ticks = self.config().max_ticks.unwrap_or(MAX_TICKS);
		self.properties.config.max_ticks = Some(max_ticks);

		while self.tick() {
			if self.tick_count().is_multiple_of(0x100) {
				clear_screen();
				print_title_short();
				eprintln!(
					"processing tick {} out of {max_ticks:0>4}",
					self.state.tick_str()
				);
				eprintln!();
			}
		}
	}
}
