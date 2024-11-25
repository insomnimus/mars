// SPDX-License-Identifier: MIT
use std::{
	env,
	ffi::OsString,
	fs,
};

pub fn get_args() -> Vec<OsString> {
	let mut args = Vec::with_capacity(16);
	let mut cli_args = env::args_os();

	let Some(config) = env::var_os("MARS_CONFIG_PATH").and_then(|p| fs::read_to_string(p).ok())
	else {
		args.extend(cli_args);
		return args;
	};

	// Argv0 is the path.
	args.extend(cli_args.next());

	args.extend(config.lines().map(str::trim).filter_map(|s| {
		if s.is_empty() || s.starts_with('#') {
			None
		} else {
			Some(s.into())
		}
	}));

	args.extend(cli_args);
	args
}
