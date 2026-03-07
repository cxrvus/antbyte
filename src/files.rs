use std::{fs, path::PathBuf, process};

use anyhow::{Context, Result, bail};

use crate::ant::{
	compiler::{LogConfig, compile_world},
	world::WorldProperties,
};

pub fn compile_world_file(path: &PathBuf, log_cfg: &LogConfig) -> Result<WorldProperties> {
	let code = read_file(path)?;
	let extension = path.extension().unwrap_or_default().to_string_lossy();

	let mut properties = match extension.as_ref() {
		"ant" => compile_world(&code, log_cfg, Some(path))
			.with_context(|| format!("compiler error in file '{}'!", path.to_string_lossy())),

		"json" => compile_json(&code),

		"mjs" => compile_mjs(path),

		_ => bail!("input files need to have a '.ant', '.json' or '.mjs' extension"),
	}?;

	if properties.name.is_none() {
		if let Some(name) = path.file_name() {
			properties.name = Some(name.to_string_lossy().to_string());
		}
	}

	Ok(properties)
}

pub fn read_file(path: &PathBuf) -> Result<String> {
	fs::read_to_string(path)
		.with_context(|| format!("error reading file '{}'!", path.to_string_lossy()))
}

pub fn compile_json(code: &str) -> Result<WorldProperties> {
	serde_json::from_str::<WorldProperties>(code).context("invalid JSON world file!")
}

pub fn compile_mjs(path: &PathBuf) -> Result<WorldProperties> {
	// idea: insert node JS warning

	let output = process::Command::new("node")
		.arg(path)
		.output()
		.context("failed to execute node-JS script!")?;

	let code = String::from_utf8(output.stdout).context("invalid UTF-8 in stdout!")?;
	let error = String::from_utf8(output.stderr).context("invalid UTF-8 in stderr!")?;

	if !output.status.success() {
		bail!("node execution failed: {error}");
	}

	serde_json::from_str::<WorldProperties>(&code).context("invalid JSON from node-JS output!")
}
