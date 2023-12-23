// SPDX-License-Identifier: Apache-2.0

use std::sync::OnceLock;

use log::{
	Level,
	LevelFilter,
	Log,
	Metadata,
	Record,
};

static LOGGER: OnceLock<Logger> = OnceLock::new();

#[derive(Debug)]
struct Logger {
	level: Level,
}

impl Log for Logger {
	fn enabled(&self, md: &Metadata) -> bool {
		md.level() <= self.level
	}

	fn log(&self, r: &Record) {
		if self.enabled(r.metadata())
			&& r.module_path().is_some_and(|m| {
				m == env!("CARGO_CRATE_NAME")
					|| m.starts_with(concat!(env!("CARGO_CRATE_NAME"), "::"))
			}) {
			match r.level() {
				Level::Error => eprintln!("error: {}", r.args()),
				Level::Warn => eprintln!("warning: {}", r.args()),
				Level::Info => println!("{}", r.args()),
				Level::Debug | Level::Trace => eprintln!("debug: {}", r.args()),
			}
		}
	}

	fn flush(&self) {}
}

pub fn init(level: Level) {
	LOGGER.set(Logger { level }).unwrap();
	log::set_logger(LOGGER.get().unwrap()).unwrap();
	log::set_max_level(LevelFilter::Debug);
}
