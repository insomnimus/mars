mod file_name;
mod logger;
mod pretty;
#[cfg(test)]
mod tests;

use std::{
	borrow::Cow::{
		self,
		Borrowed,
	},
	collections::{
		btree_map::Entry,
		BTreeMap,
	},
	fs::{
		self,
		File,
	},
	io::{
		self,
		Read,
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
use log::{
	info,
	Level,
};
use normpath::{
	BasePath,
	BasePathBuf,
};
use pulldown_cmark::{
	html,
	Event,
	Options,
	Parser,
	Tag,
};
use serde::Deserialize;
use tidier::FormatOptions;

use self::{
	file_name::FileName,
	pretty::FormatArg,
};

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
	#[arg(required_unless_present = "help_format")]
	path: Vec<PathBuf>,

	/// Do not ignore hidden files and directories while converting directories
	#[arg(short, long)]
	all: bool,

	#[command(flatten)]
	opts: RenderOptions,

	/// Do not pretty format output
	#[arg(short = 'F', long)]
	no_format: bool,

	/// Set a formatting option using the `key=value` or `key:value` syntax
	#[arg(short = 'f', long, value_parser = FormatArg::parse)]
	format: Vec<FormatArg>,

	/// Display a list of formatting options for use with --format
	#[arg(long)]
	help_format: bool,

	/// Be more verbose
	#[arg(short, long)]
	verbose: bool,
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
	/// Do not convert URL's that end with .md (effective only while converting
	/// a directory)
	#[arg(short = 'U', long)]
	no_convert_urls: bool,
	/// Convert URL's starting with / as well (root is considered the path
	/// specified with --out-dir) (effective only while converting a directory)
	#[arg(long)]
	convert_base_urls: bool,
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
	fn new<F>(html: &'b mut String, source: &str, opts: &'a RenderOptions, map: F) -> Self
	where
		F: FnMut(Event) -> Event,
	{
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

		to_html(html, body, hard_breaks, map);

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

pub struct Buffer {
	// Stores the original markdown input or the formatted final output
	pub buf: String,
	// original -> rendered body
	pub body: String,
	// body -> askama template
	pub rendered: String,
}

impl Buffer {
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		Self {
			buf: String::new(),
			body: String::new(),
			rendered: String::new(),
		}
	}

	pub fn read_file(&mut self, p: &Path) -> Result<()> {
		self.buf.clear();
		todo!("fstat the file and allocate enough space");
		File::open(p)
			.and_then(|mut f| f.read_to_string(&mut self.buf))
			.map_err(|e| anyhow!("failure reading file {}: {}", p.display(), e))?;
		Ok(())
	}

	fn render<F>(&mut self, ro: &RenderOptions, fo: Option<&FormatOptions>, map: F) -> Result<&str>
	where
		F: FnMut(Event) -> Event,
	{
		self.rendered.clear();
		self.body.clear();
		Doc::new(&mut self.body, &self.buf, ro, map).render_into(&mut self.rendered)?;

		if let Some(fo) = fo {
			self.buf.clear();
			self.rendered.push('\0');
			tidier::format_to(&self.rendered, &mut self.buf, false, fo)
				.map_err(|e| anyhow!("formatting error: {e}"))?;

			Ok(&self.buf)
		} else {
			Ok(&self.rendered)
		}
	}
}

#[cfg(windows)]
fn is_illegal_filepath(s: &str) -> bool {
	s.bytes()
		.any(|b| b <= 31 || matches!(b, b'"' | b'<' | b'>' | b'|' | b':' | b'*' | b'?' | b'\\'))
}

fn to_html<F>(buf: &mut String, md: &str, hard_breaks: bool, mut map: F)
where
	F: FnMut(Event) -> Event,
{
	let parser = Parser::new_ext(md, Options::all()).map(|e| match e {
		Event::SoftBreak if hard_breaks => Event::HardBreak,
		other => map(other),
	});
	html::push_html(buf, parser);
}

/// Splits the given url at a query or fragment, returning the slice before and
/// after as a tuple.
fn split_url(url: &str) -> (&str, &str) {
	let base = url.split('#').next().unwrap_or(url);

	// If base's last component has a ?, split at that instead
	// let (without_last, last) = base.rsplit_once('/').unwrap_or((base, ""));
	let last_slash = base.rfind('/').unwrap_or(0);
	match base[last_slash..].find('?') {
		None => (base, &url[base.len()..]),
		Some(i) => (&url[..i + last_slash], &url[i + last_slash..]),
	}
}

/// `root` must be canonicalized.
fn is_in_dir(root: &BasePath, file_path: &Path, url: &str) -> bool {
	#[cfg(not(windows))]
	if url.contains(':') {
		return false;
	}
	#[cfg(windows)]
	if is_illegal_filepath(url) {
		return false;
	}

	let Ok(file_path) = BasePath::new(file_path) else {
		return false;
	};
	let Ok(Some(parent)) = file_path.parent() else {
		return false;
	};

	let target = url
		.strip_prefix('/')
		.map(|rest| root.join(rest))
		.unwrap_or_else(|| parent.join(url));

	target
		.normalize()
		.is_ok_and(|target| target.starts_with(root) && target.is_file())
}

fn has_hidden(url: &str) -> bool {
	url.split('/').any(|s| match s.as_bytes() {
		b".." | b"." => false,
		[b'.', ..] => true,
		_ => false,
	})
}

fn run() -> Result<()> {
	#[cfg(debug_assertions)]
	{
		use clap::CommandFactory;
		Cmd::command().debug_assert();
	}
	let c = Cmd::parse();

	if c.help_format {
		pretty::show_help();
		return Ok(());
	}

	logger::init(if c.verbose { Level::Info } else { Level::Warn });

	let fo = (!c.no_format).then(|| {
		let mut fo = FormatOptions::default();
		for o in &c.format {
			o.apply(&mut fo);
		}
		fo
	});

	if let Some(dir) = &c.out_dir {
		if c.path.len() == 1 && c.path[0].is_dir() {
			convert_dir(dir, &c.path[0], !c.all, &c.opts, fo.as_ref())
		} else {
			convert_all(dir, &c.path, &c.opts, fo.as_ref())
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

		let mut buf = Buffer {
			rendered: String::with_capacity(usize::max(data.len(), 4 << 10)),
			body: String::with_capacity(usize::max(data.len(), 4 << 10)),
			buf: data,
		};

		let html = buf.render(&c.opts, fo.as_ref(), |x| x)?;
		match c.out.as_ref() {
			Some(p) if p.as_os_str() != "-" => fs::write(p, html)?,
			_ => print!("{html}"),
		}
		Ok(())
	}
}

fn convert_all(
	dir: &Path,
	files: &[PathBuf],
	ro: &RenderOptions,
	fo: Option<&FormatOptions>,
) -> Result<()> {
	// Check that no duplicate file names exist
	let mut names = BTreeMap::new();

	for path in files {
		let p = BasePathBuf::new(path)
			.map_err(|e| anyhow!("failed to canonicalize the path {}: {}", path.display(), e))?;
		let name = FileName::new(path)?;
		match names.entry(name) {
			Entry::Vacant(x) => {
				x.insert(p);
			}
			Entry::Occupied(x) if x.get() == &p => (),
			Entry::Occupied(x) => bail!(
				"duplicate file names:\n- {}\n- {}",
				x.get().as_path().display(),
				p.as_path().display()
			),
		}
	}

	fs::create_dir_all(dir)?;
	let mut buf = Buffer::new();

	for (name, p) in &names {
		let p = p.as_path();
		let mut out = dir.join(name.name());
		out.set_extension("html");

		buf.read_file(p)?;
		let html = buf.render(ro, fo, |x| x)?;
		fs::write(&out, html).map_err(|e| anyhow!("rendering to {} failed: {}", p.display(), e))?;

		info!("{}", out.display());
	}

	Ok(())
}

fn convert_dir(
	out: &Path,
	dir: &Path,
	skip_hidden: bool,
	ro: &RenderOptions,
	fo: Option<&FormatOptions>,
) -> Result<()> {
	fs::create_dir_all(out)?;
	let dir = BasePathBuf::new(dir)?;

	let mut buf = Buffer::new();

	for entry in WalkDir::new(&dir)
		.skip_hidden(skip_hidden)
		.into_iter()
		.flatten()
		.filter(|x| x.file_type.is_file() && x.file_name.as_encoded_bytes().ends_with(b".md"))
	{
		let p = entry.path();

		let mut to =
			out.join(p.strip_prefix(&dir).map_err(|e| {
				anyhow!("error constructing target path for {}: {}", p.display(), e)
			})?);
		to.set_extension("html");

		buf.read_file(&p)?;

		if let Some(parent) = to.parent() {
			fs::create_dir_all(parent)
				.map_err(|e| anyhow!("failed to create directory {}: {}", parent.display(), e))?;
		}

		let html = buf.render(ro, fo, |event| match event {
			_ if ro.no_convert_urls => event,
			Event::Start(Tag::Link(typ, dest, title))
				if ro.convert_base_urls || !dest.starts_with('/') =>
			{
				let (url, rest) = split_url(&dest);
				if (skip_hidden && has_hidden(url)) || !is_in_dir(&dir, &p, url) {
					return Event::Start(Tag::Link(typ, dest, title));
				}
				match url.strip_suffix(".md") {
					Some(without_ext) => Event::Start(Tag::Link(
						typ,
						format!("{without_ext}.html{rest}").into(),
						title,
					)),
					_ => Event::Start(Tag::Link(typ, dest, title)),
				}
			}
			_ => event,
		})?;

		fs::write(&to, html).map_err(|e| anyhow!("error rendering to {}: {}", to.display(), e))?;
		info!("{}", to.display());
	}

	Ok(())
}

fn main() {
	if let Err(e) = run() {
		eprintln!("error: {e:?}");
		process::exit(1);
	}
}
