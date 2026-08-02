use std::{
	io::{Read, stdin},
	process::Command,
};

use crate::{ui::chars_to_input, world::config::WorldConfig};

pub fn get_keys(config: &WorldConfig) -> u16 {
	if let Some(bindings) = &config.keys {
		let input_str = if config.fps.is_none() {
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

		chars_to_input(Some(bindings), &input_str)
	} else {
		0
	}
}
