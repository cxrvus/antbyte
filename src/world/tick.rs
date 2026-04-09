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

		self.event_in = self.event_out;
		self.event_out = 0;

		for i in 0..self.ants.len() {
			if self.ants[i].is_alive() {
				self.tick_ant(i);
			}
		}

		// idea: optimize defragmentation
		self.ants.iter_mut().for_each(|ant| ant.grow_up());
		self.ants.retain(|ant| ant.is_alive());

		// idea: optimize decay
		if self.config().decay.is_some() {
			self.cell_decay();
		}

		let no_ants = self.ants.is_empty();

		let tick_overflow = self
			.config()
			.ticks
			.map(|max| self.tick_count >= max)
			.unwrap_or_default();

		!(no_ants || tick_overflow)
	}
}
