use anyhow::anyhow;

pub mod gui;
pub mod raw;

pub fn chars_to_input(key_spec: Option<&str>, pressed_keys: &str) -> u16 {
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

#[derive(Debug, Clone, Default)]
pub enum RenderMode {
	#[default]
	Gui,
	Raw,
	Term,
	None,
	Gif,
}

impl TryFrom<String> for RenderMode {
	type Error = anyhow::Error;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		match value.as_str() {
			"gui" => Ok(RenderMode::Gui),
			"raw" => Ok(RenderMode::Raw),
			"term" => Ok(RenderMode::Term),
			"none" => Ok(RenderMode::None),
			"gif" => Ok(RenderMode::Gif),
			other => Err(anyhow!("invalid render mode: {other}")),
		}
	}
}
