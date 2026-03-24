use super::World;
use crate::plugins::{
	PluginSet,
	render::term_render::{clear_screen, print_title_short},
};
use anyhow::Result;

const MAX_TICKS: u32 = 1 << 16;

impl World {
	fn init(&self, plugins: &mut PluginSet) {
		plugins.renderer.open(self.config());
		plugins.ext_input.open(self.config());
		plugins.ext_output.open(self.config());
	}

	fn end(&self, plugins: &mut PluginSet) {
		plugins.renderer.close();
		plugins.ext_input.close();
		plugins.ext_output.close();
	}

	pub fn run(&mut self, plugins: &mut PluginSet) -> Result<()> {
		self.init(plugins);

		if self.config().looping {
			let properties = self.properties.clone();
			loop {
				// todo: add break condition
				let mut world = World::new(properties.clone())?;
				world.run_once(plugins);
			}
		} else {
			self.run_once(plugins);
		}

		self.end(plugins);

		Ok(())
	}

	fn run_once(&mut self, plugins: &mut PluginSet) {
		if self.config().speed.is_some() {
			plugins.renderer.render(self);

			loop {
				self.ext_input = plugins.ext_input.frame(self.config());

				let world_active = self.frame_tick();

				plugins.renderer.render(self);

				plugins.ext_output.frame(self.config(), &self.ext_output);

				self.ext_output.clear();

				if !world_active {
					break;
				}
			}
		} else {
			self.instant_run()
		}

		plugins.renderer.render(self);
	}

	fn instant_run(&mut self) {
		let max_ticks = self.config().ticks.unwrap_or(MAX_TICKS);
		self.properties.config.ticks = Some(max_ticks);

		while self.tick() {
			if self.tick_count.is_multiple_of(0x100) {
				clear_screen();
				print_title_short();
				eprintln!("processing tick {} out of {max_ticks:0>4}", self.tick_str());
				eprintln!();
			}
		}
	}
}
