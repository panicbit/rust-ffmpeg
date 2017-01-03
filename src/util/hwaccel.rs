use ffi::*;
use std::ptr;
use std::ffi::CStr;
use codec;
use util::format;
use util::media;

/// A registred hardware accelerator.
#[derive(Copy,Clone)]
pub struct HWAccel {
    ptr: *mut AVHWAccel,
}

impl HWAccel {
    /// Get an iterater over all registred hardware accelators.
    pub fn registred() -> HWAccelIter {
        HWAccelIter::new()
    }

    /// Get the name of the hardware accelated codec.
    /// The name is globally unique among encoders and among decoders
    /// (but an encoder and a decoder can share the same name).
    pub fn name(&self) -> &CStr {
        unsafe {
            CStr::from_ptr((*self.as_ptr()).name)
        }
    }

    /// Get the type of codec implemented by the hardware accelerator.
    pub fn kind(&self) -> media::Type {
        unsafe {
            media::Type::from((*self.as_ptr()).kind)
        }
    }

    /// Get the codec implemented by the hardware accelerator.
    pub fn codec_id(&self) -> codec::Id {
        unsafe {
            codec::Id::from((*self.as_ptr()).id)
        }
    }

    /// Get the supported pixel format.
    pub fn pixel_format(&self) -> format::Pixel {
        unsafe {
            format::Pixel::from((*self.as_ptr()).pix_fmt)
        }
    }
}

impl HWAccel {
    pub unsafe fn wrap(ptr: *mut AVHWAccel) -> Self {
        HWAccel { ptr: ptr }
    }

    pub unsafe fn as_ptr(&self) -> *const AVHWAccel {
        self.ptr as *const _
    }

    pub unsafe fn as_mut_ptr(&mut self) -> *mut AVHWAccel {
        self.ptr
    }
}

pub struct HWAccelIter {
    hwaccel: *mut AVHWAccel,
}

impl HWAccelIter {
    fn new() -> Self{
        unsafe {
            HWAccelIter {
                hwaccel: av_hwaccel_next(ptr::null())
            }
        }
    }
}

impl Iterator for HWAccelIter {
    type Item = HWAccel;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.hwaccel.is_null() {
                None
            } else {
                let ret = self.hwaccel;
                self.hwaccel = av_hwaccel_next(ret);
                Some(HWAccel::wrap(ret))
            }
        }
    }
}
