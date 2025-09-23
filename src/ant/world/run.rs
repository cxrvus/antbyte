use super::World;
use std::{
	io::{self, Write},
	thread,
	time::{Duration, Instant},
};

const MAX_TICKS: u32 = 1 << 16;

impl World {
	pub fn run(&mut self) {
		self.render();

		if self.config().tpf.is_some() {
			let mut last_frame = Instant::now();

			let frame_ms = match self.config().fps {
				Some(0) => panic!(),
				Some(fps) => Some(1000 / fps),
				None => None,
			};

			while self.frame_tick() {
				if let Some(frame_ms) = frame_ms {
					// wait for frame interval to elapse
					let elapsed = last_frame.elapsed().as_millis() as u32;
					if elapsed < frame_ms {
						// add a small buffer to prevent flickering
						let sleep_ms = (frame_ms - elapsed).max(1);
						thread::sleep(Duration::from_millis(sleep_ms as u64));
					}
					last_frame = Instant::now();
				} else {
					// wait for key input to continue
					println!("<i> Press <Enter> to step to next frame");
					let mut input = String::new();
					io::stdin().read_line(&mut input).unwrap();
				}

				self.render()
			}
		} else {
			self.instant_run()
		}

		self.render();
	}

	fn instant_run(&mut self) {
		let max_ticks = self.config().ticks.unwrap_or(MAX_TICKS);
		self.properties.config.ticks = Some(max_ticks);

		while self.tick() {
			print!("{}", title());
			print!("processing tick {} out of {max_ticks:0>4}", self.tick_str());
			println!();
			clear_screen();
		}
	}

	fn frame_tick(&mut self) -> bool {
		for _ in 0..self.config().tpf.unwrap() {
			if !self.tick() {
				return false;
			}
		}

		true
	}

	fn render(&self) {
		// pre-render
		let title = title();
		let world = self.color_render();
		let frame = self.tick_str();

		// print
		clear_screen();
		println!();
		println!("{title}");
		println!();
		println!("{world}\n");
		println!();
		println!("{frame}");
		println!("\n\n");

		io::stdout().flush().unwrap();
	}

	#[inline]
	fn tick_str(&self) -> String {
		format!("{:0>8}", self.tick_count())
	}

	fn color_render(&self) -> String {
		let cells = &self.cells;
		let mut string = String::new();

		for (i, cell) in cells.values.iter().enumerate() {
			if i % cells.width == 0 {
				string.push('\n');
			}

			let pos = cells.get_pos(i).unwrap();
			let ant = self.ants().iter().find(|ant| ant.pos == pos);

			match ant {
				None => {
					string.push_str(&color_cell(*cell, "  "));
				}
				Some(ant) => {
					let (char1, char2) = ant.dir_vec().principal_chars();
					let ant_chars = format!("{char1}{char2}");
					string.push_str(&color_cell(*cell, &ant_chars));
				}
			}
		}

		string
	}
}

#[inline]
fn clear_screen() {
	print!("\x1B[2J\x1B[1;1H");
}

fn title() -> String {
	r#"
░░      ░░░   ░░░  ░░        ░░       ░░░  ░░░░  ░░        ░░        ░
▒  ▒▒▒▒  ▒▒    ▒▒  ▒▒▒▒▒  ▒▒▒▒▒  ▒▒▒▒  ▒▒▒  ▒▒  ▒▒▒▒▒▒  ▒▒▒▒▒  ▒▒▒▒▒▒▒
▓  ▓▓▓▓  ▓▓  ▓  ▓  ▓▓▓▓▓  ▓▓▓▓▓       ▓▓▓▓▓    ▓▓▓▓▓▓▓  ▓▓▓▓▓      ▓▓▓
█        ██  ██    █████  █████  ████  █████  ████████  █████  ███████
█  ████  ██  ███   █████  █████       ██████  ████████  █████        █
                                                                                                                                                      
	"#
	.into()
}

fn color_codes(value: u8) -> (u8, u8) {
	let color = value & 0b0111;
	let intensity = (value & 0b1000) != 0;
	let bg_color = if intensity { 100 + color } else { 40 + color };
	let flipped_color = color ^ 0b0111;

	let fg_color = if intensity {
		90 + flipped_color
	} else {
		30 + flipped_color
	};

	(bg_color, fg_color)
}

fn color_cell(value: u8, content: &str) -> String {
	let (bg, fg) = color_codes(value);
	format!("\x1b[{fg};{bg}m{content}\x1b[0m")
}
