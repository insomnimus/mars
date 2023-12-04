use std::{
	ffi::OsStr,
	path::Path,
};

use anyhow::{
	anyhow,
	Result,
};

#[cfg_attr(not(windows), derive(Eq, PartialEq, Ord, PartialOrd))]
pub struct FileName<'a> {
	name: &'a OsStr,
	#[cfg(windows)]
	upper: String,
}

impl<'a> FileName<'a> {
	pub fn new(p: &'a Path) -> Result<Self> {
		let name = p
			.file_name()
			.ok_or_else(|| anyhow!("unable to determine the file name for path {}", p.display()))?;
		Ok(Self {
			name,
			#[cfg(windows)]
			upper: name
				.to_str()
				.ok_or_else(|| {
					anyhow!("unable to determine the file name for path {}", p.display())
				})?
				.to_uppercase(),
		})
	}

	pub fn name(&self) -> &'a OsStr {
		self.name
	}
}

#[cfg(windows)]
mod win_impls {
	use std::cmp::Ordering;

	use super::FileName;

	impl<'a> PartialEq for FileName<'a> {
		fn eq(&self, rhs: &Self) -> bool {
			self.upper == rhs.upper
		}
	}

	impl<'a> Eq for FileName<'a> {}

	impl<'a> PartialOrd for FileName<'a> {
		fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
			Some(self.cmp(rhs))
		}
	}

	impl<'a> Ord for FileName<'a> {
		fn cmp(&self, rhs: &Self) -> Ordering {
			self.upper.cmp(&rhs.upper)
		}
	}
}
