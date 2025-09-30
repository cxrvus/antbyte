use std::{thread, time::Duration};

pub mod command_parser;
pub mod interrupt;
mod watch;

#[inline]
pub fn clear_screen() {
	print!("\x1B[2J\x1B[1;1H");
}

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
pub fn print_title_short() {
	println!("<<ANTBYTE>>");
}

#[inline]
pub fn print_error(e: anyhow::Error) {
	// need to conventionally make all anyhow context messages end in a '!'
	eprintln!("{}", format!("<!> {e:#}").replace("!: ", ":\n    "));
}

#[inline]
pub fn sleep(ms: u32) {
	thread::sleep(Duration::from_millis(ms as u64));
}
