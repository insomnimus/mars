# Mars
Mars is a Markdown to HTML convertion tool.

## Features
- Supports github flavoured markdown
- Can convert an entire directory, preserving the filesystem hierarchy
- Self contained and lean executable
- Lets you insert custom CSS, scripts or raw HTML into `<head>`
- Minimal memory footprint

## Installation
Grab a binary from [the releases page](https://github.com/insomnimus/mars/releases) ([here's the latest release](https://github.com/insomnimus/mars/releases/latest)) and put it somewhere in your PATH.

Or build from source:
## Building From Source
You need a working rost toolchain version 1.64.0 or newer.

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
- Multiple input files, write to directory: Covnerts all input files and writes under the output directory: `mars foo.md bar.md -O ../docs`

Additionally you can insert styling, scripts or otherwise any raw HTML into the <head>` section of converted documents.
### Example: Use Sakura CSS
```shell
mars ./docs/ -O ./docs/html -c "https://cdn.jsdelivr.net/npm/sakura.css/css/sakura.css"
# You can specify the -c option multiple times
```

### Example: Convert all Markdown files in your home
```shell
mars "$HOME" -O ./docs
```

### Example: Convert a single file
```shell
mars ./readme.md -o ./readme.html
```
