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
