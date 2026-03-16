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

		"js" | "mjs" => compile_js(path),

		_ => bail!(
			"invalid file extension: {extension}.\n needs to be either: '.ant', '.json', '.js', '.mjs'"
		),
	}?;

	if properties.name.is_none()
		&& let Some(name) = path.file_name()
	{
		properties.name = Some(name.to_string_lossy().to_string());
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

pub fn compile_js(path: &PathBuf) -> Result<WorldProperties> {
	// idea: insert node JS warning

	let output = process::Command::new("node")
		.arg(path)
		.output()
		.context("failed to execute nodejs script!")?;

	let code = String::from_utf8(output.stdout).context("invalid UTF-8 in stdout!")?;
	let error = String::from_utf8(output.stderr).context("invalid UTF-8 in stderr!")?;

	if !output.status.success() {
		bail!("nodejs execution failed: {error}");
	}

	serde_json::from_str::<WorldProperties>(&code).context("invalid JSON from nodejs output!")
}
