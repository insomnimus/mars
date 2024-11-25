# Formatting Options
You can specify formatting options on the command line using the `-f` or `--format` options.

The syntax is `option=value`, or `option:value`.
The value will be trimmed of leading and trailling whitespace.

Options and values are case insensitive.

For example:
```shell
mars input.md -f indent=tabs -f wrap=100
```

Boolean options can be set to `on` without specifying the value:
```shell
mars input.md -fremove-comments
```

Boolean options accept...
- true, t, on, yes, 1
- false, f, off, no, 0

# Available Options

## eol
The line ending to use.
- type: enum(lf|crlf|cr)
- default: lf

## indent
The indentation style.
A value of 0 disables indentation.
- type: an integer for number of spaces, "tabs" for tabs
- default: 4

## indent-attributes
Put a newline and indent after each attribute.
- type: bool
- default: off

## indent-cdata
Indent contents of `<![CDATA[]]]>` tags.
- type: bool
- default: off

## join-classes
Join multiple class attributes into one within a tag.
- type: bool
- default: off

## join-styles
Join multiple style attributes into one within a tag.
- type: bool
- default: off

## merge-divs
Merge multiple `<div>` elements.
- type: bool
- default: off

## merge-spans
Merge multiple `<span>` elements.
- type: bool
- default: off

## newline-after-br
Put a newline after `<br>` tags.
- type: bool
- default: off

## remove-comments
Remove all comments from the document.
- type: bool
- default: off

## wrap
The maximum length of a line.
Lines will be wrapped after the threshold if possible.

A value of 0 disables line wrapping.
- type: integer
- default: 68
