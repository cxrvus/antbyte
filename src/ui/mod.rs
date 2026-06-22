pub mod term;

pub fn chars_to_input(key_spec: &Option<String>, pressed_keys: &str) -> u8 {
	if let Some(key_spec) = key_spec {
		let mut value = 0;

		for (i, key) in key_spec.char_indices() {
			if pressed_keys.contains(key) {
				value |= 1 << i;
			}
		}
		value
	} else {
		0
	}
}
