use std::{io::stdin, process::Command};

use crate::world::config::WorldConfig;

pub fn get_keys(config: &WorldConfig) -> u8 {
	if let Some(bindings) = &config.keys {
		// temporary convention: begin key binding string with a SPACE to select synchronous input mode
		let input_str = if !bindings.starts_with(" ") {
			// cross platform mode...

			eprintln!("<i> Press <Enter> to send input");
			let mut input_str = String::new();
			stdin().read_line(&mut input_str).unwrap();
			input_str
		} else {
			eprintln!("<i> Input Mode");

			// linux-specific mode...

			if !cfg!(target_os = "linux") {
				panic!(
					"Input Mode is only supported on Linux. Set your K0 binding to a SPACE to use the cross-platform Input Mode"
				);
			}

			let saved_mode = Command::new("stty")
				.arg("-g")
				.output()
				.ok()
				.and_then(|out| String::from_utf8(out.stdout).ok())
				.map(|s| s.trim().to_string());
			let _ = Command::new("stty")
				.args(["-icanon", "-echo", "min", "1", "time", "0"])
				.status();

			let mut b = [0u8; 1];
			let key_in = if stdin().read_exact(&mut b).is_ok() {
				Some(b[0] as char)
			} else {
				None
			};

			if let Some(mode) = saved_mode {
				let _ = Command::new("stty").arg(mode).status();
			} else {
				let _ = Command::new("stty").arg("sane").status();
			}

			String::from(key_in.unwrap_or(' '))
		};

		let mut input_val = 0u8;

		for key_in in input_str.chars() {
			for (i, binding) in bindings.chars().enumerate() {
				if key_in == binding {
					input_val |= 1 << i;
				}
			}
		}

		input_val
	} else {
		0
	}
}
