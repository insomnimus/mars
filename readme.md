# Mars
Mars is a Markdown to HTML convertion tool.

## Features
- Supports github flavoured markdown
- Supports metadata blocks
- Can convert an entire directory, preserving the filesystem hierarchy
- While converting a directory, convert relative markdown links in documents to .html if the specified file exists
- Self contained and lean executable
- Automatic pretty formatting of generated HTML through statically linked [libtidy](https://github.com/htacg/tidy-html5)
- Lets you insert custom CSS, scripts or raw HTML into `<head>`
- Minimal memory footprint

## Installation
Grab a binary from [the releases page](https://github.com/insomnimus/mars/releases) ([here's the latest release](https://github.com/insomnimus/mars/releases/latest)) and put it somewhere in your PATH.

Or build from source:
## Building From Source
You will need:
- A working rust toolchain version 1.74.0 or newer
- clang, for generating libtidy bindings on the go
- CMake and a C compiler, to build libtidy from source

you don't need libtidy installed. [tidy-sys](https://github.com/insomnimus/tidy-sys) takes care of it automatically.

```shell
# The file will be located in `target/release/mars` (with a .exe suffix on Windows)
# You can move it anywhere you wish
cargo build --release
# OR you can install through cargo
cargo install --path .
```

## Usage
There are 4 modes of operation:
- Single input file, no output file: prints to stdout: `mars foo.md`
- Single input file, one output file: Converts input and saves to output: `mars foo.md -o foo.html`
- Single input directory, write to directory: Converts `.md` files in input recursively and writes to output directory, preserving the hierarchy: `mars . -O ../docs` (notice the capital `-O`)
- Multiple input files, write to directory: Converts all input files and writes under the output directory: `mars foo.md bar.md -O ../docs`

Additionally you can insert styling, scripts or otherwise any raw HTML into the `<head>` section of converted documents.
### Example: Use Sakura CSS
```shell
mars ./docs/ -O ./docs/html -c "https://cdn.jsdelivr.net/npm/sakura.css/css/sakura.css"
# You can specify the -c option multiple times
# There's a convenience flag for Sakura.css:
mars ./docs/ -O ./docs/html --sakura-css
```

### Example: Convert all Markdown files in your home
```shell
# The -a/--all flag makes mars not ignore hidden files and folders
mars "$HOME" --all -O ./docs
```

### Example: Convert a single file
```shell
mars ./readme.md -o ./readme.html
```

### Example: Read markdown from stdin
```shell
cat readme.md | mars -o readme.html -
```

## Metadata
You can specify metadata on top of a markdown file. The format for the block is YAML.

The below snippet demonstrates the usage and all the possible metadata fields:
```markdown
---
title: Example Metadata Usage
lang: en
hard_breaks: true
css: ["https://example.com/foo.css", "https://example.com/bar.css"]
script: ["https://example.com/foo.js"]
head: '<meta name="description" content="Demonstrate usage of metadata blocks!">'
---

Rest of your content goes here.
```

That is
- A metadata block starts with `---`
- and then a new line
- then a YAML map containing key-value pairs
- and it ends with a line containing only `---`

Keys not shown above are simply ignored.

If the text between `---` does not contain valid YAML, it is not considered a metadata block; it's rendered to HTML as Markdown.

### Metadata Precedence
- For keys that are not lists, values in the Markdown source take precedence.
- For lists such as `css` and `script`, the values are combined into a single list without duplication
	- Insertion order is preserved.
	- The values specified on the command line are appended to values specified in the source.
	- If the `--normalize-css` flag is used, the `Normalize.css` import will be put on top.
	- If the `--sakura-css` flag is used, the `Sakura.css` import will be last.
