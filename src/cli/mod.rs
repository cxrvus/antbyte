pub mod command_parser;

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

pub fn print_title_short() {
	println!("<<ANTBYTE>>");
}
