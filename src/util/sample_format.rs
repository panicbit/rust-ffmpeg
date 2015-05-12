use std::ffi::{CStr, CString};
use std::str::from_utf8_unchecked;
use std::ops::Index;
use std::ptr;
use std::slice;
use std::mem;

use libc::{c_int, uint8_t};
use ffi::*;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum SampleFormat {
	None,
	U8,
	S16,
	S32,
	FLT,
	DBL,

	U8P,
	S16P,
	S32P,
	FLTP,
	DBLP,
}

impl SampleFormat {
	pub fn name(&self) -> &'static str {
		unsafe {
			from_utf8_unchecked(CStr::from_ptr(av_get_sample_fmt_name((*self).into())).to_bytes())
		}
	}

	pub fn packed(&self) -> Self {
		unsafe {
			SampleFormat::from(av_get_packed_sample_fmt((*self).into()))
		}
	}

	pub fn planar(&self) -> Self {
		unsafe {
			SampleFormat::from(av_get_planar_sample_fmt((*self).into()))
		}
	}

	pub fn is_planar(&self) -> bool {
		unsafe {
			av_sample_fmt_is_planar((*self).into()) == 1
		}
	}

	pub fn bytes(&self) -> usize {
		unsafe {
			av_get_bytes_per_sample((*self).into()) as usize
		}
	}

	pub fn buffer(&self, channels: usize, samples: usize, align: bool) -> Buffer {
		Buffer::new(*self, channels, samples, align)
	}
}

impl From<AVSampleFormat> for SampleFormat {
	fn from(value: AVSampleFormat) -> Self {
		match value {
			AV_SAMPLE_FMT_NONE => SampleFormat::None,
			AV_SAMPLE_FMT_U8   => SampleFormat::U8,
			AV_SAMPLE_FMT_S16  => SampleFormat::S16,
			AV_SAMPLE_FMT_S32  => SampleFormat::S32,
			AV_SAMPLE_FMT_FLT  => SampleFormat::FLT,
			AV_SAMPLE_FMT_DBL  => SampleFormat::DBL,

			AV_SAMPLE_FMT_U8P  => SampleFormat::U8P,
			AV_SAMPLE_FMT_S16P => SampleFormat::S16P,
			AV_SAMPLE_FMT_S32P => SampleFormat::S32P,
			AV_SAMPLE_FMT_FLTP => SampleFormat::FLTP,
			AV_SAMPLE_FMT_DBLP => SampleFormat::DBLP,

			AV_SAMPLE_FMT_NB => SampleFormat::None
		}
	}
}

impl From<&'static str> for SampleFormat {
	fn from(value: &'static str) -> Self {
		unsafe {
			SampleFormat::from(av_get_sample_fmt(CString::new(value).unwrap().as_ptr()))
		}
	}
}

impl Into<AVSampleFormat> for SampleFormat {
	fn into(self) -> AVSampleFormat {
		match self {
			SampleFormat::None => AV_SAMPLE_FMT_NONE,
			SampleFormat::U8   => AV_SAMPLE_FMT_U8,
			SampleFormat::S16  => AV_SAMPLE_FMT_S16,
			SampleFormat::S32  => AV_SAMPLE_FMT_S32,
			SampleFormat::FLT  => AV_SAMPLE_FMT_FLT,
			SampleFormat::DBL  => AV_SAMPLE_FMT_DBL,

			SampleFormat::U8P  => AV_SAMPLE_FMT_U8P,
			SampleFormat::S16P => AV_SAMPLE_FMT_S16P,
			SampleFormat::S32P => AV_SAMPLE_FMT_S32P,
			SampleFormat::FLTP => AV_SAMPLE_FMT_FLTP,
			SampleFormat::DBLP => AV_SAMPLE_FMT_DBLP,
		}
	}
}

pub struct Buffer {
	pub format: SampleFormat,
	pub channels: usize,
	pub samples: usize,
	pub align: bool,

	buffer: *mut *mut uint8_t,
	size:   c_int,
}

impl Buffer {
	pub fn size(format: SampleFormat, channels: usize, samples: usize, align: bool) -> usize {
		unsafe {
			av_samples_get_buffer_size(ptr::null_mut(), channels as c_int, samples as c_int, format.into(), !align as c_int) as usize
		}
	}

	pub fn new(format: SampleFormat, channels: usize, samples: usize, align: bool) -> Self {
		unsafe {
			let mut buf = Buffer {
				format:   format,
				channels: channels,
				samples:  samples,
				align:    align,

				buffer: ptr::null_mut(),
				size:   0,
			};

			av_samples_alloc_array_and_samples(&mut buf.buffer, &mut buf.size,
			                 channels as c_int, samples as c_int,
			                 format.into(), !align as c_int);

			buf
		}
	}
}

impl Index<usize> for Buffer {
	type Output = [u8];

	fn index<'a>(&'a self, index: usize) -> &'a [u8] {
		if index >= self.samples {
			panic!("out of bounds");
		}

		unsafe {
			slice::from_raw_parts(*self.buffer.offset(index as isize), self.size as usize)
		}
	}
}

impl Clone for Buffer {
	fn clone(&self) -> Self {
		let mut buf = Buffer::new(self.format, self.channels, self.samples, self.align);
		buf.clone_from(self);

		buf
	}

	fn clone_from(&mut self, source: &Self) {
		unsafe {
			av_samples_copy(self.buffer, mem::transmute(source.buffer), 0, 0, source.samples as c_int, source.channels as c_int, source.format.into());
		}
	}
}

impl Drop for Buffer {
	fn drop(&mut self) {
		unsafe {
			av_freep(mem::transmute(self.buffer));
		}
	}
}
