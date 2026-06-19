use crate::{
	util::{dir::Direction, vec2::Pos},
	world::{
		config::{RenderMask, WorldConfig},
		frame::FrameOutput,
	},
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

impl TermRenderer {
	pub fn render_frame(&self, frame: &FrameOutput) {
		let world_str = self.render_cells(frame);

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

	fn render_cells(&self, frame: &FrameOutput) -> String {
		let mut string = String::new();
		let max_index = self.config.height as usize * self.config.width as usize;

		for i in 0..max_index {
			let pos = Pos::from_index(i, self.config.width);

			if pos.x == 0 {
				string.push('\n');
			}

			let cell_color = frame.bg.get(&pos).unwrap_or(&0);
			let fg_value = frame.fg.get(&pos);

			let cell_text = match fg_value {
				None => "  ",
				Some(&fg_value) => match self.config.fg {
					RenderMask::Dir => {
						let dir = Direction::from(fg_value);
						let (char1, char2) = dir.as_chars();
						&format!("{char1}{char2}")
					}
					RenderMask::None => "  ",
					_ => &format!("{fg_value:02X}"),
				},
			};

			string.push_str(&render_cell(*cell_color, cell_text));
		}

		string
	}
}

fn render_cell(color: u8, text: &str) -> String {
	let (bg, fg) = color_codes(color);
	format!("\x1b[{fg};{bg}m{text}\x1b[0m")
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
