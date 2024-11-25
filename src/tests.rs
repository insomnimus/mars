// SPDX-License-Identifier: MIT

use super::*;

#[test]
fn test_split_url() {
	let tests = [
		("a/b", "a/b", ""),
		("a/b#c", "a/b", "#c"),
		("a/b?c", "a/b", "?c"),
		("a/b?c#d", "a/b", "?c#d"),
		("a/b#c?d", "a/b", "#c?d"),
		("a/b?c/d", "a/b?c/d", ""),
		("a/b#c#d/e#g", "a/b", "#c#d/e#g"),
		// Without separators
		("a", "a", ""),
		("a#b", "a", "#b"),
		("a?b", "a", "?b"),
		("a?b#c", "a", "?b#c"),
		("a#b?c", "a", "#b?c"),
		("a#b#c", "a", "#b#c"),
	];

	for (s, a, b) in tests {
		let got = split_url(s);
		assert_eq!((a, b), got, "\ninput: {s}");
	}
}

#[test]
fn test_has_hidden() {
	let test_no = [
		"",
		".",
		"..",
		"/",
		"a",
		"a/b",
		"./a",
		"./a/..",
		"../a",
		".././a",
		"a.b/c..d/",
		"a/../b/./c",
		"./../..",
	];

	let test_yes = [".a", "..a", "./.a", "../.a", "a.b/.c", "/.a"];

	for s in test_no {
		assert!(
			!has_hidden(s),
			"\nisn't hidden but found to be hidden:\ninput: {s}"
		);
	}
	for s in test_yes {
		assert!(
			has_hidden(s),
			"\nis hidden but found to be not:\ninput: {s}"
		);
	}
}
