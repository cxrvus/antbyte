use super::Renderer;
use crate::{util::sleep, world::World};

use std::{
	io::{self, Write},
	time::Instant,
};

#[inline]
pub fn print_title() {
	let title = r#"
░░      ░░░   ░░░  ░░        ░░       ░░░  ░░░░  ░░        ░░        ░
▒  ▒▒▒▒  ▒▒    ▒▒  ▒▒▒▒▒  ▒▒▒▒▒  ▒▒▒▒  ▒▒▒  ▒▒  ▒▒▒▒▒▒  ▒▒▒▒▒  ▒▒▒▒▒▒▒
▓  ▓▓▓▓  ▓▓  ▓  ▓  ▓▓▓▓▓  ▓▓▓▓▓       ▓▓▓▓▓    ▓▓▓▓▓▓▓  ▓▓▓▓▓      ▓▓▓
█        ██  ██    █████  █████  ████  █████  ████████  █████  ███████
█  ████  ██  ███   █████  █████       ██████  ████████  █████        █
                                                                                                                                                      
	"#;

	println!("{title}");
}

#[inline]
#[rustfmt::skip]
pub fn clear_screen() { print!("\x1B[2J\x1B[1;1H"); }

#[inline]
#[rustfmt::skip]
pub fn print_title_short() { println!("<<ANTBYTE>>"); }

pub struct TermRenderer {
	last_frame: Instant,
	frame_ms: Option<u32>,
	cfg_sleep: Option<u32>,
}

impl Renderer for TermRenderer {
	fn render(&mut self, world: &World) {
		// wait before every frame (except frame 0)
		if world.tick_count() > 0 {
			if let Some(frame_ms) = self.frame_ms {
				// wait for frame interval to elapse
				let elapsed = self.last_frame.elapsed().as_millis() as u32;
				if elapsed < frame_ms {
					// add a small buffer to prevent flickering
					let sleep_ms = (frame_ms - elapsed).max(8);
					sleep(sleep_ms);
				}
				self.last_frame = Instant::now();
			} else {
				// wait for key input to continue
				eprintln!("<i> Press <Enter> to step to next frame");
				let mut input = String::new();
				io::stdin().read_line(&mut input).unwrap();
			}
		}

		self.render_frame(world);
	}

	fn open(&mut self) {
		clear_screen();
		println!();
		sleep(100);
	}

	fn close(&self) {
		if let Some(ms) = self.cfg_sleep {
			sleep(ms);
		}
	}
}

impl TermRenderer {
	pub fn new(world: &World) -> Self {
		let frame_ms = match world.config().fps {
			Some(0) => panic!(),
			Some(fps) => Some(1000 / fps),
			None => None,
		};

		Self {
			last_frame: Instant::now(),
			cfg_sleep: world.config().sleep,
			frame_ms,
		}
	}

	fn render_frame(&self, world: &World) {
		// pre-render
		let world_str = self.color_render(world);
		let tick_str = world.tick_str();

		// print
		clear_screen();

		if !world.config().hide_title {
			print_title();
		}

		if let Some(name) = &world.name() {
			println!("{name}\n");
		}

		println!("\n\n{world_str}\n\n");
		println!("{tick_str}\n\n\n");

		io::stdout().flush().unwrap();
	}

	fn color_render(&self, world: &World) -> String {
		let cells = &world.cells;
		let mut string = String::new();

		for (i, cell) in cells.entries.iter().enumerate() {
			if i % cells.width == 0 {
				string.push('\n');
			}

			let pos = cells.get_pos(i).unwrap();
			let ant = world.ants().iter().find(|ant| ant.pos == pos);

			match ant {
				None => {
					string.push_str(&color_cell(cell.value, "  "));
				}
				Some(ant) => {
					let (char1, char2) = ant.dir_vec().principal_chars();
					let ant_chars = format!("{char1}{char2}");
					string.push_str(&color_cell(cell.value, &ant_chars));
				}
			}
		}

		string
	}
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
