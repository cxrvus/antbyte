use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};

use crate::ant::{
	compiler::{LogConfig, compile_world},
	world::WorldProperties,
};

pub fn compile_world_file(path: &PathBuf, log_cfg: &LogConfig) -> Result<WorldProperties> {
	let code = read_file(path)?;
	let extension = path.extension().unwrap_or_default().to_string_lossy();

	match extension.as_ref() {
		"ant" => compile_world(&code, log_cfg, Some(path))
			.with_context(|| format!("compiler error in file '{}'!", path.to_string_lossy())),

		"json" => compile_json(&code),

		_ => bail!("input files need to have a '.ant', '.json' or '.js' extension"),
	}
}

pub fn read_file(path: &PathBuf) -> Result<String> {
	fs::read_to_string(path)
		.with_context(|| format!("error reading file '{}'!", path.to_string_lossy()))
}

pub fn compile_json(code: &str) -> Result<WorldProperties> {
	serde_json::from_str::<WorldProperties>(code).context("invalid JSON world file")
}
