use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};

use crate::ant::{
	compiler::{LogConfig, compile_world},
	world::WorldProperties,
};

pub fn compile_world_file(path: &PathBuf, log_cfg: &LogConfig) -> Result<WorldProperties> {
	let code = read_file(path)?;

	let extension = path.extension().unwrap_or_default().to_string_lossy();

	if extension != "ant" {
		bail!("ant files need to have a '.ant' extension");
	}

	compile_world(&code, log_cfg, Some(path))
		.with_context(|| format!("compiler error in file '{}'!", path.to_string_lossy()))
}

pub fn read_file(path: &PathBuf) -> Result<String> {
	fs::read_to_string(path)
		.with_context(|| format!("error reading file '{}'!", path.to_string_lossy()))
}
