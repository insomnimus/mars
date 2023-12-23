// SPDX-License-Identifier: Apache-2.0

#[cfg(not(windows))]
pub use non_win::FileName;
#[cfg(windows)]
pub use win::FileName;

#[cfg(windows)]
mod win {
	use std::{
		cmp::Ordering,
		ffi::{
			OsStr,
			OsString,
		},
		os::windows::ffi::{
			OsStrExt,
			OsStringExt,
		},
		path::Path,
		ptr::addr_of_mut,
	};

	use anyhow::{
		anyhow,
		Result,
	};
	use windows::{
		core::PCWSTR,
		Win32::{
			Foundation::INVALID_HANDLE_VALUE,
			Storage::FileSystem::{
				FindClose,
				FindFirstFileW,
				WIN32_FIND_DATAW,
			},
		},
	};

	pub struct FileName {
		name: OsString,
		upper: String,
	}

	impl FileName {
		pub fn new(p: &Path) -> Result<Self> {
			let name = real_name(p)
				.or_else(|| p.file_name().map(|name| name.to_os_string()))
				.ok_or_else(|| {
					anyhow!("unable to determine the file name for path {}", p.display())
				})?;

			Ok(Self {
				upper: name
					.to_str()
					.ok_or_else(|| {
						anyhow!("unable to determine the file name for path {}", p.display())
					})?
					.to_uppercase(),
				name,
			})
		}

		pub fn name(&self) -> &OsStr {
			&self.name
		}
	}

	impl PartialEq for FileName {
		fn eq(&self, rhs: &Self) -> bool {
			self.upper == rhs.upper
		}
	}

	impl Eq for FileName {}

	impl PartialOrd for FileName {
		fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
			Some(self.cmp(rhs))
		}
	}

	impl Ord for FileName {
		fn cmp(&self, rhs: &Self) -> Ordering {
			self.upper.cmp(&rhs.upper)
		}
	}

	fn real_name(p: &Path) -> Option<OsString> {
		if p.as_os_str()
			.as_encoded_bytes()
			.iter()
			.any(|&b| b == b'?' || b == b'*')
		{
			return None;
		}
		let mut s = Vec::with_capacity(p.as_os_str().len() + 2);
		s.extend(p.as_os_str().encode_wide());
		s.push(0);

		let mut data = WIN32_FIND_DATAW::default();

		unsafe {
			match FindFirstFileW(PCWSTR(s.as_ptr()), addr_of_mut!(data)) {
				Ok(h) if h != INVALID_HANDLE_VALUE => {
					let _ = FindClose(h);
					Some(OsString::from_wide(
						data.cFileName
							.split(|&c| c == 0)
							.next()
							.unwrap_or(&data.cFileName),
					))
				}
				_ => None,
			}
		}
	}
}

#[cfg(not(windows))]
mod non_win {
	use std::{
		ffi::OsStr,
		path::Path,
	};

	use anyhow::{
		anyhow,
		Result,
	};

	#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
	pub struct FileName<'a> {
		name: &'a OsStr,
	}

	impl<'a> FileName<'a> {
		pub fn new(p: &'a Path) -> Result<Self> {
			p.file_name()
				.map(|name| Self { name })
				.ok_or_else(|| anyhow!("failed to determine the file name for {}", p.display()))
		}

		pub fn name(self) -> &'a OsStr {
			self.name
		}
	}
}
