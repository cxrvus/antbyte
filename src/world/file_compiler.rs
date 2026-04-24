use std::{
	fs,
	io::{IsTerminal, read_to_string, stdin},
	path::PathBuf,
	process,
};

use anyhow::{Context, Result, bail};

use crate::{
	parser::compiler::{LogConfig, compile_world as compile_dot_ant},
	world::WorldProperties,
};

pub fn compile_world(
	path: &PathBuf,
	log_cfg: &LogConfig,
	sub_args: &Option<String>,
) -> Result<WorldProperties> {
	let file_name = path
		.file_name()
		.unwrap_or_default()
		.to_string_lossy()
		.to_string();

	let mut properties = if file_name.starts_with(".") {
		if stdin().is_terminal() {
			bail!("path '.ant' / '.json' requires text being piped via stdin");
		}

		let code = read_to_string(stdin()).context("error reading from stdin!")?;

		match file_name.as_ref() {
			".ant" => compile_dot_ant(&code, log_cfg, Some(path)).with_context(|| "compiler error"),
			".json" => compile_json(&code),
			_ => bail!("can only use .ant or .json files using pipes"),
		}
	} else {
		let code = read_file(path)?;

		let extension = path
			.extension()
			.unwrap_or_default()
			.to_string_lossy()
			.to_string();

		match extension.as_ref() {
			"ant" => compile_dot_ant(&code, log_cfg, Some(path))
				.with_context(|| format!("compiler error in file '{}'!", path.to_string_lossy())),

			"json" => compile_json(&code),

			"js" | "mjs" => compile_js(path, sub_args.clone()),

			_ => bail!(
				"invalid file extension: {extension}.\n needs to be either: '.ant', '.json', '.js', '.mjs'"
			),
		}
	}?;

	if properties.name.is_none() && !file_name.starts_with(".") {
		properties.name = Some(file_name);
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

pub fn compile_js(path: &PathBuf, args: Option<String>) -> Result<WorldProperties> {
	// idea: insert node JS warning

	let output = process::Command::new("node")
		.arg(path)
		.arg(args.unwrap_or_default())
		.output()
		.context("failed to execute nodejs script!")?;

	let code = String::from_utf8(output.stdout).context("invalid UTF-8 in stdout!")?;
	let error = String::from_utf8(output.stderr).context("invalid UTF-8 in stderr!")?;

	if !output.status.success() {
		bail!("nodejs execution failed: {error}");
	}

	if !error.trim().is_empty() {
		eprintln!("nodejs stderr output:");
		eprintln!("{error}");
	}

	serde_json::from_str::<WorldProperties>(&code).context("invalid JSON from nodejs output!")
}
