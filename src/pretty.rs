// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::type_complexity)]

use tidier::{
	FormatOptions,
	LineEnding,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum IndentStyle {
	Tabs,
	Spaces(u16),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FormatArg {
	Indent(IndentStyle),
	IndentAttributes(bool),
	IndentCdata(bool),
	Wrap(u32),
	RemoveComments(bool),
	Eol(LineEnding),
	JoinClasses(bool),
	JoinStyles(bool),
	NewlineAfterBr(bool),
	MergeDivs(bool),
	MergeSpans(bool),
}

fn error(arg: &str, val: &str, msg: &str) -> String {
	format!("invalid value '{val}' for `{arg}`: {msg}")
}

impl FormatArg {
	pub fn parse(s: &str) -> Result<Self, String> {
		use FormatArg::*;

		let (orig_arg, orig_val) = s.split_once([':', '=']).unwrap_or((s, ""));

		let orig_arg = orig_arg.trim();
		let arg = orig_arg.to_lowercase();
		let orig_val = orig_val.trim();
		let val = orig_val.to_lowercase();

		const BOOLS: &[(&str, fn(bool) -> FormatArg)] = &[
			("indent-attributes", IndentAttributes),
			("indent-cdata", IndentCdata),
			("remove-comments", RemoveComments),
			("join-classes", JoinClasses),
			("join-styles", JoinStyles),
			("newline-after-br", NewlineAfterBr),
			("merge-divs", MergeDivs),
			("merge-spans", MergeSpans),
		];

		match arg.as_str() {
			"indent" => match val.as_str() {
				"tabs" | "tab" => Ok(Self::Indent(IndentStyle::Tabs)),
				"" | "spaces" | "space" => Ok(Self::Indent(IndentStyle::Spaces(4))),
				_ => val
					.parse::<u16>()
					.map(|n| Self::Indent(IndentStyle::Spaces(n)))
					.map_err(|_| {
						error(
							&arg,
							orig_val,
							"value must be 'tabs', 'spaces' or a non-negative integer",
						)
					}),
			},
			"wrap" => val
				.parse::<u32>()
				.map(Self::Wrap)
				.map_err(|_| error(&arg, orig_val, "value must be a non-negative integer")),
			"eol" | "newline" => match val.as_str() {
				"lf" => Ok(Self::Eol(LineEnding::Lf)),
				"crlf" => Ok(Self::Eol(LineEnding::CrLf)),
				"cr" => Ok(Self::Eol(LineEnding::Cr)),
				_ => Err(error(
					&arg,
					orig_val,
					"value must be one of 'lf', 'crlf' and 'cr'",
				)),
			},
			_ => {
				for &(name, f) in BOOLS {
					if arg == name {
						if val.is_empty() {
							return Ok(f(true));
						}
						return match val.to_lowercase().as_str() {
							"true" | "t" | "on" | "yes" | "1" => Ok(f(true)),
							"false" | "f" | "off" | "no" | "0" => Ok(f(false)),
							_ => Err(error(&arg, orig_val, "value must be boolean")),
						};
					}
				}

				Err(format!("unknown format option `{orig_arg}`"))
			}
		}
	}

	pub fn apply(self, o: &mut FormatOptions) {
		use FormatArg::*;

		match self {
			Indent(IndentStyle::Tabs) => {
				o.indent.tabs = true;
				o.indent.size = 8;
			}
			Indent(IndentStyle::Spaces(n)) => {
				o.indent.tabs = false;
				o.indent.size = n;
			}
			IndentAttributes(x) => o.indent.attributes = x,
			IndentCdata(x) => o.indent.cdata = x,
			Wrap(x) => o.wrap = x,
			RemoveComments(x) => o.strip_comments = x,
			Eol(x) => o.eol = x,
			JoinClasses(x) => o.join_classes = x,
			JoinStyles(x) => o.join_styles = x,
			NewlineAfterBr(x) => o.br_newline = x,
			MergeDivs(x) => o.merge_divs = x,
			MergeSpans(x) => o.merge_spans = x,
		}
	}
}

pub fn show_help() {
	println!("{}", include_str!("../docs/formatting-options.md").trim());
}
