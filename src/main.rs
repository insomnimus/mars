use std::{
	fs::{
		self,
		File,
	},
	io::{
		self,
		BufWriter,
		Read,
		Write,
	},
	path::Path,
	process,
};

use anyhow::{
	anyhow,
	bail,
	Result,
};
use askama::Template;
use clap::Parser as ArgParser;
use jwalk::WalkDir;
use pulldown_cmark::{
	html,
	Options,
	Parser,
};

#[derive(ArgParser)]
#[command(version)]
/// Converts Markdown files into HTML
struct Cmd {
	/// Write output to a file
	#[arg(short, long, group = "output")]
	out: Option<String>,
	/// Write all converted html files into a directory
	#[arg(short = 'O', long, group = "output")]
	out_dir: Option<String>,

	/// Path to a single directory or one or more markdown files
	#[arg(required = true)]
	path: Vec<String>,

	#[command(flatten)]
	opts: RenderOptions,
}

#[derive(ArgParser)]
struct RenderOptions {
	/// Import CSS styles from a URL
	#[arg(short, long)]
	css: Vec<String>,
	/// Import a script from a URL
	#[arg(short, long)]
	script: Vec<String>,

	/// Append raw HTML into <head>
	#[arg(short = 'H', long, default_value_t = String::new(), hide_default_value = true)]
	head: String,
}

#[derive(Template)]
#[template(path = "template.html")]
struct Doc<'o, 'b> {
	opts: &'o RenderOptions,
	body: &'b str,
}

fn run() -> Result<()> {
	let c = Cmd::parse();
	if let Some(dir) = &c.out_dir {
		convert_to_dir(dir, &c.opts, &c.path)
	} else if c.path.len() != 1 {
		bail!("cannot write multiple fiels into one; use the --out-dir option instead");
	} else {
		let data = if &c.path[0] == "-" {
			let mut buf = String::with_capacity(4 << 10);
			io::stdin().lock().read_to_string(&mut buf)?;
			buf
		} else {
			fs::read_to_string(&c.path[0])?
		};

		let mut file;
		let mut stdout;
		let out: &mut dyn Write = match c.out.as_deref() {
			Some("-") | None => {
				stdout = io::stdout().lock();
				&mut stdout
			}
			Some(p) => {
				file = File::create(p)?;
				&mut file
			}
		};

		let mut body = String::with_capacity(8 << 10);
		to_html(&mut body, &data);

		let doc = Doc {
			opts: &c.opts,
			body: &body,
		};
		let mut out = BufWriter::new(out);
		doc.write_into(&mut out)?;
		out.flush()?;
		Ok(())
	}
}

fn to_html(buf: &mut String, md: &str) {
	let parser = Parser::new_ext(md, Options::all());
	html::push_html(buf, parser);
}

fn convert_to_dir<D: AsRef<Path>, F: AsRef<Path>>(
	dir: D,
	opts: &RenderOptions,
	files: &[F],
) -> Result<()> {
	let dir = dir.as_ref();
	fs::create_dir_all(dir)?;

	// If files contains a single directory, recursively find .md files, preserving
	// the filesystem hierarchy
	if files.len() == 1 && files[0].as_ref().is_dir() {
		convert_dir(dir, opts, files[0].as_ref())?;
	} else {
		// Else write all files to dir
		let mut buf = String::with_capacity(8 << 10);
		for p in files.iter().map(AsRef::as_ref) {
			let mut out = dir
				.join(p.file_name().ok_or_else(|| {
					anyhow!("cannot determine the file name for {}", p.display())
				})?);
			out.set_extension("html");
			let data = fs::read_to_string(p)
				.map_err(|e| anyhow!("failkure reading file {}: {}", p.display(), e))?;
			let mut file = BufWriter::new(
				File::create(&out)
					.map_err(|e| anyhow!("failure writing to {}: {}", out.display(), e))?,
			);

			buf.clear();
			to_html(&mut buf, &data);
			let doc = Doc { opts, body: &buf };
			doc.write_into(&mut file)?;
			file.flush()?;
			println!("{}", out.display());
		}
	}

	Ok(())
}

fn convert_dir(out: &Path, opts: &RenderOptions, dir: &Path) -> Result<()> {
	let mut buf = String::with_capacity(8 << 10);
	for entry in WalkDir::new(dir)
		.into_iter()
		.flatten()
		.filter(|x| x.file_type().is_file())
	{
		let p = entry.path();
		let mut to =
			out.join(p.strip_prefix(dir).map_err(|e| {
				anyhow!("error constructing target path for {}: {}", p.display(), e)
			})?);
		to.set_extension("html");
		let data = fs::read_to_string(&p)
			.map_err(|e| anyhow!("failure reading {}: {}", p.display(), e))?;

		if let Some(parent) = to.parent() {
			fs::create_dir_all(parent)
				.map_err(|e| anyhow!("failed to create directory {}: {}", parent.display(), e))?;
		}

		let mut file = BufWriter::new(
			File::create(&to).map_err(|e| anyhow!("failed to write to {}: {}", to.display(), e))?,
		);

		buf.clear();
		to_html(&mut buf, &data);

		let doc = Doc { opts, body: &buf };
		doc.write_into(&mut file)?;
		file.flush()?;

		println!("{}", to.display());
	}

	Ok(())
}

fn main() {
	if let Err(e) = run() {
		eprintln!("error: {e:?}");
		process::exit(1);
	}
}
