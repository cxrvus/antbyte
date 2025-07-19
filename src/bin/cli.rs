use std::io::{self, IsTerminal, Read};

fn main() {
	if io::stdin().is_terminal() {
		eprintln!("<!> no pipe input detected. please pipe data into this command.");
		std::process::exit(1);
	}

	println!("<<ANTBYTE>>\n");

	let mut buffer = String::new();

	match io::stdin().read_to_string(&mut buffer) {
		Ok(_) => println!("{buffer}"),
		Err(e) => {
			eprintln!("error reading from stdin: {e}");
			std::process::exit(1);
		}
	}
}
