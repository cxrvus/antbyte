#![cfg(feature = "extras")]

use crate::cli::{interrupt, sleep};

use super::{
	command_parser::{Args, run_once},
	print_error,
};

use anyhow::Result;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc;
use std::time::Duration;

pub fn watch_file(args: &mut Args) -> Result<()> {
	let (tx, rx) = mpsc::channel();

	let mut watcher = RecommendedWatcher::new(
		move |res: Result<Event, _>| {
			if let Ok(event) = res {
				let _ = tx.send(event);
			}
		},
		Config::default(),
	)?;

	eprintln!("watching file: {:?}", args.path);
	watcher.watch(&args.path, RecursiveMode::NonRecursive)?;
	args.watch = false;

	let _handle = std::thread::spawn({
		let args = args.clone();
		move || {
			if let Err(e) = run_once(args) {
				print_error(e);
			}
		}
	});

	let mut pending_change = false;

	loop {
		match rx.recv_timeout(Duration::from_millis(100)) {
			Ok(event) => {
				if event.kind.is_modify()
					&& event
						.paths
						.iter()
						.any(|p| p.canonicalize().is_ok_and(|cp| cp == args.path))
				{
					pending_change = true;
				}
			}
			Err(mpsc::RecvTimeoutError::Timeout) => {
				if pending_change {
					pending_change = false;

					interrupt::enable_interrupt();
					sleep(200);
					interrupt::disable_interrupt();

					let _handle = std::thread::spawn({
						let args = args.clone();
						move || {
							if let Err(e) = run_once(args) {
								print_error(e);
							}
						}
					});
				}
			}
			Err(_) => break,
		}
	}

	Ok(())
}
