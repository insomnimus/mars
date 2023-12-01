use std::{
	borrow::Cow::{
		self,
		Borrowed,
	},
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
	path::{
		Path,
		PathBuf,
	},
	process,
};

use anyhow::{
	anyhow,
	bail,
	Result,
};
use askama::Template;
use clap::Parser as ArgParser;
use indexmap::IndexSet;
use jwalk::WalkDir;
use pulldown_cmark::{
	html,
	Event,
	Options,
	Parser,
};
use serde::Deserialize;

#[derive(ArgParser)]
#[command(version)]
/// Converts Markdown files into HTML
struct Cmd {
	/// Write output to a file
	#[arg(short, long, group = "output")]
	out: Option<PathBuf>,
	/// Write all converted html files into a directory
	#[arg(short = 'O', long, group = "output")]
	out_dir: Option<PathBuf>,

	/// Path to a single directory or one or more markdown files
	#[arg(required = true)]
	path: Vec<PathBuf>,

	/// Do not ignore hidden files and directories
	#[arg(short, long)]
	all: bool,

	#[command(flatten)]
	opts: RenderOptions,
}

#[derive(ArgParser)]
struct RenderOptions {
	/// Set the lang attribute of <html>
	#[arg(short, long)]
	lang: Option<String>,

	/// Import CSS styles from a URL
	#[arg(short, long)]
	css: Vec<String>,
	/// Import a script from a URL
	#[arg(short, long)]
	script: Vec<String>,

	/// Import Normalize.css
	#[arg(short = 'N', long)]
	normalize_css: bool,
	/// Import Sakura.css
	#[arg(short = 'S', long)]
	sakura_css: bool,

	/// Append raw HTML into <head>
	#[arg(long, default_value_t = String::new(), hide_default_value = true)]
	head: String,

	/// Turn newlines into hard breaks
	#[arg(short = 'H', long)]
	hard_breaks: bool,
}

#[derive(Template)]
#[template(path = "template.html")]
struct Doc<'b, 'o> {
	md: Metadata<'o>,
	body: &'b str,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Metadata<'a> {
	title: Option<Cow<'a, str>>,
	lang: Option<Cow<'a, str>>,
	css: IndexSet<Cow<'a, str>>,
	script: IndexSet<Cow<'a, str>>,
	head: Cow<'a, str>,
	hard_breaks: Option<bool>,
}

impl<'b, 'a> Doc<'b, 'a> {
	fn new(html: &'b mut String, source: &str, opts: &'a RenderOptions) -> Self {
		const WHITESPACE: &[char] = &[' ', '\t', '\n', '\r'];
		let s = source.trim_start_matches(WHITESPACE);

		let (mut md, body) =
			s.strip_prefix("---")
				.filter(|s| s.starts_with('\n') || s.starts_with("\r\n"))
				.and_then(|s| {
					s.split_once("\n---").filter(|(_, body)| {
						body.starts_with('\n')
							|| body.starts_with("\r\n") || body.trim_matches(WHITESPACE).is_empty()
					})
				})
				.and_then(|(md, body)| {
					serde_yaml::from_str::<Metadata>(md)
						.ok()
						.map(|md| (md, body))
				})
				.unwrap_or_else(|| (Metadata::default(), s.trim_matches(WHITESPACE)));

		let hard_breaks = md.hard_breaks.unwrap_or(opts.hard_breaks);

		to_html(html, body, hard_breaks);

		// Put normalize.css on top
		if opts.normalize_css {
			md.css.insert(Borrowed(
				"https://unpkg.com/normalize.css@8.0.1/normalize.css",
			));
			md.css.move_index(md.css.len() - 1, 0);
		}

		md.css.extend(opts.css.iter().map(|x| Borrowed(x.as_str())));
		md.script
			.extend(opts.script.iter().map(|x| Borrowed(x.as_str())));

		if md.head.is_empty() {
			md.head = Borrowed(opts.head.as_str());
		}
		if md.lang.is_none() {
			md.lang = opts.lang.as_deref().map(Borrowed);
		}

		if opts.sakura_css {
			md.css.insert(Borrowed(
				"https://cdn.jsdelivr.net/npm/sakura.css/css/sakura.css",
			));
		}

		Self {
			md,
			body: html.trim_matches(WHITESPACE),
		}
	}
}

fn to_html(buf: &mut String, md: &str, hard_breaks: bool) {
	let parser = Parser::new_ext(md, Options::all()).map(|e| match e {
		Event::SoftBreak if hard_breaks => Event::HardBreak,
		other => other,
	});
	html::push_html(buf, parser);
}

fn run() -> Result<()> {
	let c = Cmd::parse();
	if let Some(dir) = &c.out_dir {
		if c.path.len() == 1 && c.path[0].is_dir() {
			convert_dir(dir, &c.opts, &c.path[0], !c.all)
		} else {
			convert_all(dir, &c.opts, &c.path)
		}
	} else if c.path.len() != 1 {
		bail!("cannot write multiple files into one; use the --out-dir option instead");
	} else {
		let data = if c.path[0].as_os_str() == "-" {
			let mut buf = String::with_capacity(8 << 10);
			io::stdin().lock().read_to_string(&mut buf)?;
			buf
		} else {
			fs::read_to_string(&c.path[0])?
		};

		let mut file;
		let mut stdout;
		let out: &mut dyn Write = match c.out.as_ref() {
			Some(p) if p.as_os_str() != "-" => {
				file = File::create(p)?;
				&mut file
			}
			_ => {
				stdout = io::stdout().lock();
				&mut stdout
			}
		};

		let mut body = String::with_capacity(8 << 10);

		let doc = Doc::new(&mut body, &data, &c.opts);
		let mut out = BufWriter::new(out);
		doc.write_into(&mut out)?;
		out.flush()?;
		Ok(())
	}
}

fn convert_all(dir: &Path, opts: &RenderOptions, files: &[PathBuf]) -> Result<()> {
	fs::create_dir_all(dir)?;
	let mut buf = String::with_capacity(8 << 10);
	let mut file_buf = String::with_capacity(8 << 10);

	for p in files.iter() {
		let mut out = dir.join(
			p.file_name()
				.ok_or_else(|| anyhow!("cannot determine the file name for {}", p.display()))?,
		);
		out.set_extension("html");

		file_buf.clear();
		File::open(p)
			.and_then(|mut f| f.read_to_string(&mut file_buf))
			.map_err(|e| anyhow!("failure reading file {}: {}", p.display(), e))?;

		let mut file = BufWriter::new(
			File::create(&out)
				.map_err(|e| anyhow!("failure writing to {}: {}", out.display(), e))?,
		);

		buf.clear();
		let doc = Doc::new(&mut buf, &file_buf, opts);
		doc.write_into(&mut file)?;
		file.flush()?;
		println!("{}", out.display());
	}

	Ok(())
}

fn convert_dir(out: &Path, opts: &RenderOptions, dir: &Path, skip_hidden: bool) -> Result<()> {
	fs::create_dir_all(out)?;
	let mut buf = String::with_capacity(8 << 10);
	let mut file_buf = String::with_capacity(8 << 10);

	for entry in WalkDir::new(dir)
		.skip_hidden(skip_hidden)
		.into_iter()
		.flatten()
		.filter(|x| x.file_type.is_file() && x.file_name.as_encoded_bytes().ends_with(b".md"))
	{
		let p = entry.path();
		let mut to =
			out.join(p.strip_prefix(dir).map_err(|e| {
				anyhow!("error constructing target path for {}: {}", p.display(), e)
			})?);
		to.set_extension("html");

		file_buf.clear();
		File::open(&p)
			.and_then(|mut f| f.read_to_string(&mut file_buf))
			.map_err(|e| anyhow!("failure reading {}: {}", p.display(), e))?;

		if let Some(parent) = to.parent() {
			fs::create_dir_all(parent)
				.map_err(|e| anyhow!("failed to create directory {}: {}", parent.display(), e))?;
		}

		let mut file = BufWriter::new(
			File::create(&to).map_err(|e| anyhow!("failed to write to {}: {}", to.display(), e))?,
		);

		buf.clear();

		let doc = Doc::new(&mut buf, &file_buf, opts);
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
