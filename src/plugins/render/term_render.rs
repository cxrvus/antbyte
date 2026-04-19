use super::Renderer;
use crate::{
	plugins::Plugin,
	util::sleep,
	world::{World, WorldState, config::WorldConfig},
};

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
	hide_title: bool,
}

impl Plugin for TermRenderer {
	fn open(&mut self, _config: &WorldConfig) {
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
}

type CellRenderer = Box<dyn Fn(u8, &str) -> String + 'static>;

impl TermRenderer {
	pub fn new(world: &World, hide_title: bool) -> Self {
		let frame_ms = match world.config().fps {
			Some(0) => panic!(),
			Some(fps) => Some(1000 / fps),
			None => None,
		};

		Self {
			last_frame: Instant::now(),
			cfg_sleep: world.config().sleep,
			frame_ms,
			hide_title,
		}
	}

	fn render_frame(&self, world: &World) {
		// pre-render
		let cell_renderer = if let Some(ascii) = &world.config().ascii {
			let palette = if ascii.is_empty() {
				ASCII_DEFAULT
			} else {
				ascii
			};
			ascii_cell(palette)
		} else {
			Box::new(color_cell)
		};

		let world_str = self.render_cells(world, &cell_renderer);

		// print
		clear_screen();

		if !self.hide_title {
			print_title();
		}

		if let Some(name) = &world.name() {
			println!("{name}\n");
		}

		println!("\n\n{world_str}\n\n");
		println!("{}", world.metadata_str());

		io::stdout().flush().unwrap();
	}

	fn render_cells(&self, world: &World, render_cell: &CellRenderer) -> String {
		let cells = &world.cells;
		let mut string = String::new();

		for (i, cell) in cells.entries.iter().enumerate() {
			if i % cells.width == 0 {
				string.push('\n');
			}

			let pos = cells.get_pos(i).unwrap();
			let ant = world.ants().iter().find(|ant| ant.pos == pos);

			let cell_value = world.adjusted_color(cell.value);

			match (ant, world.config().hide_ants) {
				(None, _) | (_, true) => {
					string.push_str(&render_cell(cell_value, "  "));
				}
				(Some(ant), false) => {
					let (char1, char2) = ant.dir.as_chars();
					let ant_chars = format!("{char1}{char2}");
					string.push_str(&render_cell(cell_value, &ant_chars));
				}
			}
		}

		string
	}
}

const ASCII_DEFAULT: &str = ".,-=+:;cna!?$W#@";

fn ascii_cell(palette: &str) -> CellRenderer {
	let palette = palette.to_string();
	Box::new(move |value: u8, content: &str| -> String {
		if !content.trim().is_empty() {
			content.into()
		} else {
			let value = value.clamp(0, 15) as usize;
			let char = &palette[value..value + 1];
			char.repeat(2)
		}
	})
}

fn color_cell(value: u8, content: &str) -> String {
	let (bg, fg) = color_codes(value);
	format!("\x1b[{fg};{bg}m{content}\x1b[0m")
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

impl WorldState {
	#[inline]
	pub fn tick_str(&self) -> String {
		format!("T: {:0>8}", self.tick_count())
	}

	pub fn ext_out_str(&self) -> String {
		let ext_out_str = self
			.ext_output
			.iter()
			.map(|x| format!("{x:02x}"))
			.collect::<Vec<_>>()
			.join(", ");

		if ext_out_str.is_empty() {
			"--".into()
		} else {
			ext_out_str
		}
	}

	pub fn metadata_str(&self) -> String {
		let tick_str = self.tick_str();
		let ext_out_str = self.ext_out_str();

		format!("{tick_str}\nK: {:02x}\nX: {ext_out_str}\n", self.ext_input)
	}
}
