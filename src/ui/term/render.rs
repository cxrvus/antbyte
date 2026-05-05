use crate::world::World;
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

pub struct TermRenderer {
	hide_title: bool,
}

type CellRenderer = Box<dyn Fn(u8, &str) -> String + 'static>;

impl TermRenderer {
	pub fn new(hide_title: bool) -> Self {
		Self { hide_title }
	}

	pub fn render_frame(&self, world: &World) {
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
			if i % cells.width as usize == 0 {
				string.push('\n');
			}

			let pos = cells.get_pos(i).unwrap();
			let ant = world.ants().get(&pos);

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
