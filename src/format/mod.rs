pub use ::util::format::{sample, Sample};
pub use ::util::format::{pixel, Pixel};

pub mod stream;

pub mod context;
pub use self::context::Context;

pub mod format;
pub use self::format::{Input, Output, list};
pub use self::format::{flag, Flags};

pub mod network;

use std::ptr;
use std::slice;
use std::io::{Seek, Read, SeekFrom};
use std::path::Path;
use std::ffi::{CString, CStr};
use std::str::from_utf8_unchecked;
use std::mem;
use std::fs::File;
use libc::{self, c_int, uint8_t, int64_t, c_void};

use ffi::*;
use ::{Error, Format, Dictionary};

pub fn register_all() {
	unsafe {
		av_register_all();
	}
}

pub fn register(format: &Format) {
	match format {
		&Format::Input(ref format) => unsafe {
			av_register_input_format(format.as_ptr());
		},

		&Format::Output(ref format) => unsafe {
			av_register_output_format(format.as_ptr());
		}
	}
}

pub fn version() -> u32 {
	unsafe {
		avformat_version()
	}
}

pub fn configuration() -> &'static str {
	unsafe {
		from_utf8_unchecked(CStr::from_ptr(avformat_configuration()).to_bytes())
	}
}

pub fn license() -> &'static str {
	unsafe {
		from_utf8_unchecked(CStr::from_ptr(avformat_license()).to_bytes())
	}
}

// XXX: use to_cstring when stable
fn from_path<P: AsRef<Path>>(path: &P) -> CString {
	CString::new(path.as_ref().as_os_str().to_str().unwrap()).unwrap()
}

// NOTE: this will be better with specialization or anonymous return types
pub fn open<P: AsRef<Path>>(path: &P, format: &Format) -> Result<Context, Error> {
	unsafe {
		let mut ps   = ptr::null_mut();
		let     path = from_path(path);

		match format {
			&Format::Input(ref format) => {
				match avformat_open_input(&mut ps, path.as_ptr(), format.as_ptr(), ptr::null_mut()) {
					0 => {
						match avformat_find_stream_info(ps, ptr::null_mut()) {
							r if r >= 0 => Ok(Context::Input(context::Input::wrap(ps))),
							e           => Err(Error::from(e)),
						}
					}

					e => Err(Error::from(e))
				}
			}

			&Format::Output(ref format) => {
				match avformat_alloc_output_context2(&mut ps, format.as_ptr(), ptr::null(), path.as_ptr()) {
					0 => {
						match avio_open(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE) {
							0 => Ok(Context::Output(context::Output::wrap(ps))),
							e => Err(Error::from(e)),
						}
					}

					e => Err(Error::from(e))
				}
			}
		}
	}
}

pub fn open_with<P: AsRef<Path>>(path: &P, format: &Format, options: Dictionary) -> Result<Context, Error> {
	unsafe {
		let mut ps   = ptr::null_mut();
		let     path = from_path(path);
		let mut opts = options.disown();

		match format {
			&Format::Input(ref format) => {
				let res = avformat_open_input(&mut ps, path.as_ptr(), format.as_ptr(), &mut opts);

				Dictionary::own(opts);

				match res {
					0 => {
						match avformat_find_stream_info(ps, ptr::null_mut()) {
							r if r >= 0 => Ok(Context::Input(context::Input::wrap(ps))),
							e           => Err(Error::from(e)),
						}
					}

					e => Err(Error::from(e))
				}
			}

			&Format::Output(ref format) => {
				match avformat_alloc_output_context2(&mut ps, format.as_ptr(), ptr::null(), path.as_ptr()) {
					0 => {
						match avio_open(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE) {
							0 => Ok(Context::Output(context::Output::wrap(ps))),
							e => Err(Error::from(e)),
						}
					}

					e => Err(Error::from(e))
				}
			}
		}
	}
}

pub fn input<P: AsRef<Path>>(path: &P) -> Result<context::Input, Error> {
	unsafe {
		let mut ps   = ptr::null_mut();
		let     path = from_path(path);

		match avformat_open_input(&mut ps, path.as_ptr(), ptr::null_mut(), ptr::null_mut()) {
			0 => {
				match avformat_find_stream_info(ps, ptr::null_mut()) {
					r if r >= 0 => Ok(context::Input::wrap(ps)),
					e           => Err(Error::from(e)),
				}
			}

			e => Err(Error::from(e))
		}
	}
}

pub fn input_with<P: AsRef<Path>>(path: &P, options: Dictionary) -> Result<context::Input, Error> {
	unsafe {
		let mut ps   = ptr::null_mut();
		let     path = from_path(path);
		let mut opts = options.disown();
		let     res  = avformat_open_input(&mut ps, path.as_ptr(), ptr::null_mut(), &mut opts);

		Dictionary::own(opts);

		match res {
			0 => {
				match avformat_find_stream_info(ps, ptr::null_mut()) {
					r if r >= 0 => Ok(context::Input::wrap(ps)),
					e           => Err(Error::from(e)),
				}
			}
			
			e => Err(Error::from(e))
		}
	}
}

/// Initialize a context with custom input instead of a file.
pub fn input_io<I: AVInput>(input: I) -> Result<context::Input, Error> {
	unsafe {
		let mut ps    = avformat_alloc_context();
		assert!(!ps.is_null(), "Could not allocate AVFormat context");

		(*ps).pb = input_into_context(input);

		match avformat_open_input(&mut ps, ptr::null(), ptr::null_mut(), ptr::null_mut()) {
			0 => {
				match avformat_find_stream_info(ps, ptr::null_mut()) {
					r if r >= 0 => Ok(context::Input::wrap(ps)),
					e           => Err(Error::from(e)),
				}
			}

			e => Err(Error::from(e))
		}
	}
}

pub fn output<P: AsRef<Path>>(path: &P) -> Result<context::Output, Error> {
	unsafe {
		let mut ps     = ptr::null_mut();
		let     path   = from_path(path);

		match avformat_alloc_output_context2(&mut ps, ptr::null_mut(), ptr::null(), path.as_ptr()) {
			0 => {
				match avio_open(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE) {
					0 => Ok(context::Output::wrap(ps)),
					e => Err(Error::from(e))
				}
			}

			e => Err(Error::from(e))
		}
	}
}

pub fn output_with<P: AsRef<Path>>(path: &P, options: Dictionary) -> Result<context::Output, Error> {
	unsafe {
		let mut ps     = ptr::null_mut();
		let     path   = from_path(path);
		let mut opts   = options.disown();

		match avformat_alloc_output_context2(&mut ps, ptr::null_mut(), ptr::null(), path.as_ptr()) {
			0 => {
				let res = avio_open2(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE, ptr::null(), &mut opts,);

				Dictionary::own(opts);

				match res {
					0 => Ok(context::Output::wrap(ps)),
					e => Err(Error::from(e))
				}
			}

			e => Err(Error::from(e))
		}
	}
}

pub fn output_as<P: AsRef<Path>>(path: &P, format: &str) -> Result<context::Output, Error> {
	unsafe {
		let mut ps     = ptr::null_mut();
		let     path   = from_path(path);
		let     format = CString::new(format).unwrap();

		match avformat_alloc_output_context2(&mut ps, ptr::null_mut(), format.as_ptr(), path.as_ptr()) {
			0 => {
				match avio_open(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE) {
					0 => Ok(context::Output::wrap(ps)),
					e => Err(Error::from(e))
				}
			}

			e => Err(Error::from(e))
		}
	}
}

pub fn output_as_with<P: AsRef<Path>>(path: &P, format: &str, options: Dictionary) -> Result<context::Output, Error> {
	unsafe {
		let mut ps     = ptr::null_mut();
		let     path   = from_path(path);
		let     format = CString::new(format).unwrap();
		let mut opts   = options.disown();

		match avformat_alloc_output_context2(&mut ps, ptr::null_mut(), format.as_ptr(), path.as_ptr()) {
			0 => {
				let res = avio_open2(&mut (*ps).pb, path.as_ptr(), AVIO_FLAG_WRITE, ptr::null(), &mut opts,);

				Dictionary::own(opts);

				match res {
					0 => Ok(context::Output::wrap(ps)),
					e => Err(Error::from(e))
				}
			}

			e => Err(Error::from(e))
		}
	}
}

pub trait AVSeek: Sized + Send + 'static {
	/// Seek to `pos`. Returns `Some(new_pos)` on success
	/// and `None` on error.
	fn seek(&mut self, pos: SeekFrom) -> Option<u64>;
	/// The size of the data. It is optional to support this.
	fn size(&self) -> Option<u64> {
		None
	}
}

/// Implementors of AVInput can be used as custom input source.
pub trait AVInput: AVSeek + Sized + Send + 'static {
	/// Fill the buffer.
	/// Returns the number of bytes read.
	/// `None` or `Some(0)` indicates **EOF**.
	fn read_packet(&mut self, buf: &mut [u8]) -> Option<usize>;
	/// The buffer size is very important for performance.
	/// For protocols with fixed blocksize it should be set to this blocksize.
	/// For others a typical size is a cache page, e.g. 4kb.
	///
	/// Default: 4kb.
	fn buffer_size() -> c_int { 4 * 1024 }
}

/// Implementors of AVOutput can be used as custom output source.
pub trait AVOutput: AVSeek + Sized + Send + 'static {
	/// Write the buffer to the output.
	/// Returns the number of bytes written.
	/// `None` or `Some(0)` indicates failure.
	fn write_packet(&mut self, buf: &[u8]) -> Option<usize>;
	/// The buffer size is very important for performance.
	/// For protocols with fixed blocksize it should be set to this blocksize.
	/// For others a typical size is a cache page, e.g. 4kb.
	///
	/// Default: 4kb.
	fn buffer_size() -> c_int { 4 * 1024 }
}

fn input_into_context<I: AVInput>(input: I) -> *mut AVIOContext  {
	unsafe {
		let buffer_size = I::buffer_size();
		let buffer = av_malloc(buffer_size as usize * mem::size_of::<uint8_t>()) as _;
		let write_flag = 0; // Make buffer read-only for ffmpeg
		let read_packet = ffi_read_packet::<I>;
		let write_packet = mem::transmute(0 as *const c_void); // should maybe be Option in ffmpeg_sys
		let seek = ffi_seek::<I>;
		let this = Box::into_raw(Box::new(input)) as *mut c_void;
		let avio_ctx = avio_alloc_context(
			buffer,
			buffer_size,
			write_flag,
			this,
			read_packet,
			write_packet,
			seek
		);

		assert!(!avio_ctx.is_null(), "Could not allocate AVIO context");

		avio_ctx
	}
}

extern fn ffi_read_packet<I: AVInput>(this: *mut c_void, buf: *mut uint8_t, buf_size: c_int) -> c_int {
	let this = unsafe { &mut *(this as *mut I) };
	let buf = unsafe { slice::from_raw_parts_mut(buf, buf_size as usize) };
	let eof = -1;
	this.read_packet(buf).map(|n_read| n_read as c_int).unwrap_or(eof)
}

extern fn ffi_seek<S: AVSeek>(this: *mut c_void, offset: int64_t, whence: c_int) -> int64_t {
	let this = unsafe { &mut *(this as *mut S) };

	// According to the doc AVSEEK_SIZE is ORed with whence.
	if offset & AVSEEK_SIZE as int64_t == AVSEEK_SIZE as int64_t {
		return this.size().and_then(u64_into_int64_t).unwrap_or(-1);
	}

	let pos = match whence {
		libc::SEEK_SET => match int64_t_into_u64(offset) {
			Some(offset) => SeekFrom::Start(offset),
			None => return -1,
		},
		libc::SEEK_CUR => SeekFrom::Current(offset),
		libc::SEEK_END => SeekFrom::End(offset),
		_ => return -1,
	};

	this.seek(pos).and_then(u64_into_int64_t).unwrap_or(-1)
}

fn u64_into_int64_t(n: u64) -> Option<int64_t> {
	if n <= int64_t::max_value() as u64 {
		Some(n as int64_t)
	} else {
		None
	}
}

fn int64_t_into_u64(n: int64_t) -> Option<u64> {
	if n >= 0 {
		Some(n as u64)
	} else {
		None
	}
}

impl AVSeek for File {
	fn seek(&mut self, pos: SeekFrom) -> Option<u64> {
		Seek::seek(self, pos).ok()
	}
	fn size(&self) -> Option<u64> {
		self.metadata().map(|m| m.len()).ok()
	}
}

impl AVInput for File {
	fn read_packet(&mut self, buf: &mut [u8]) -> Option<usize> {
		self.read(buf).ok()
	}
}
