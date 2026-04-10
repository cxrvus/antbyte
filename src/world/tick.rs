use crate::world::World;

impl World {
	pub fn frame_tick(&mut self) -> bool {
		for _ in 0..self
			.config()
			.speed
			.expect("speed must be greater than 0 to use frame_tick")
		{
			if !self.tick() {
				return false;
			}
		}

		true
	}

	pub fn tick(&mut self) -> bool {
		self.tick_count += 1;

		// events
		self.event_in = self.event_out;
		self.event_out = 0;

		// tick ants
		let image = self.state.clone();
		let all_outputs: Vec<_> = image.ants.iter().map(|ant| self.get_output(ant)).collect();

		for (i, p) in all_outputs.iter().enumerate() {
			// SYNC
			self.tick_ant(i, p, Self::sync_tick);

			// KILL
			self.tick_ant(i, p, Self::kill_tick);
			self.clean_up_ants();

			//TODO: CONTINUE: the former approach doesn't work with stages that require conflict resolution
			// TODO: move

			// TODO: spawn

			// TODO: die

			self.clean_up_ants();
		}

		// idea: optimize decay
		// cell decay
		if self.config().decay.is_some() {
			self.cell_decay();
		}

		// end world if conditions are met
		let no_ants = self.ants.is_empty();

		let tick_overflow = self
			.config()
			.ticks
			.map(|max| self.tick_count >= max)
			.unwrap_or_default();

		!(no_ants || tick_overflow)
	}
}
