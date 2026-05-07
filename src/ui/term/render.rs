use crate::{
	util::dir::Direction,
	world::{config::WorldConfig, frame::FrameOutput},
};
use std::io::{self, Write};

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

pub(super) struct TermRenderer {
	pub(super) hide_title: bool,
	pub(super) config: WorldConfig,
	pub(super) name: Option<String>,
}

type CellRenderer = Box<dyn Fn(u8, &str) -> String + 'static>;

impl TermRenderer {
	pub fn render_frame(&self, frame: &FrameOutput) {
		// pre-render
		let cell_renderer = if let Some(ascii) = &self.config.ascii {
			let palette = if ascii.is_empty() {
				ASCII_DEFAULT
			} else {
				ascii
			};
			ascii_cell(palette)
		} else {
			Box::new(color_cell)
		};

		let world_str = self.render_cells(frame, &cell_renderer);

		// print
		clear_screen();

		if !self.hide_title {
			print_title();
		}

		if let Some(name) = &self.name {
			println!("{name}\n");
		}

		println!("\n\n{world_str}\n\n");
		println!("{}", frame.metadata);

		io::stdout().flush().unwrap();
	}

	fn render_cells(&self, frame: &FrameOutput, render_cell: &CellRenderer) -> String {
		let mut string = String::new();

		for (i, &bg_value) in frame.bg.entries.iter().enumerate() {
			if i % self.config.width as usize == 0 {
				string.push('\n');
			}

			let fg_value = frame.fg.get(&frame.bg.pos_from_index(i).unwrap());

			match (fg_value, self.config.hide_ants) {
				(None, _) | (_, true) => {
					string.push_str(&render_cell(bg_value, "  "));
				}
				(Some(&fg_value), false) => {
					// todo: implement using render settings
					let dir = Direction::from(fg_value);
					let (char1, char2) = dir.as_chars();
					let ant_chars = format!("{char1}{char2}");
					string.push_str(&render_cell(bg_value, &ant_chars));
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
