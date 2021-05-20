//! Simple writer to file descriptor using libc.
//!
//! ## Features:
//!
//! - `std` - Enables `std::io::Write` implementation.
//!

#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

use core::{slice, cmp, mem, ptr, fmt};

const BUFFER_CAPACITY: usize = 4096;

///Wrapper into file descriptor.
pub struct FdWriter {
    fd: libc::c_int,
    len: u16,
    buffer: mem::MaybeUninit<[u8; BUFFER_CAPACITY]>,
}

impl FdWriter {
    ///Creates new instance which writes into `fd`
    pub const fn new(fd: libc::c_int) -> Self {
        Self {
            fd,
            len: 0,
            buffer: mem::MaybeUninit::uninit(),
        }
    }

    #[inline(always)]
    ///Returns pointer to first element in underlying buffer.
    pub const fn as_ptr(&self) -> *const u8 {
        &self.buffer as *const _ as *const _
    }

    #[inline(always)]
    ///Returns pointer to first element in underlying buffer.
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buffer.as_mut_ptr() as *mut _ as *mut _
    }

    #[inline]
    ///Returns immutable slice with current elements
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self.as_ptr(), self.len as _)
        }
    }

    fn inner_flush(&mut self) {
        let text = unsafe {
            core::str::from_utf8_unchecked(self.as_slice())
        };
        unsafe {
            libc::write(self.fd.into(), text.as_ptr() as *const _, text.len() as _);
        }
        self.len = 0;
    }

    ///Flushes buffer, clearing buffer.
    pub fn flush(&mut self) {
        if self.len > 0 {
            self.inner_flush();
        }
    }

    #[inline]
    fn copy_data<'a>(&mut self, data: &'a [u8]) -> &'a [u8] {
        let write_len = cmp::min(BUFFER_CAPACITY.saturating_sub(self.len as _), data.len());
        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), self.as_mut_ptr().add(self.len as _), write_len);
        }
        self.len += write_len as u16;
        &data[write_len..]
    }

    ///Writes data unto buffer.
    ///
    ///Flushing if it ends with `\n` automatically
    pub fn write_data(&mut self, mut data: &[u8]) {
        loop {
            data = self.copy_data(data);

            if data.len() == 0 {
                break;
            } else {
                self.flush();
            }
        }

        if self.as_slice()[self.len as usize - 1] == b'\n' {
            self.flush();
        }
    }
}

impl fmt::Write for FdWriter {
    #[inline]
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.write_data(text.as_bytes());

        Ok(())
    }
}

#[cfg(feature = "std")]
impl std::io::Write for FdWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write_data(buf);
        Ok(buf.len())
    }

    #[inline(always)]
    fn flush(&mut self) -> std::io::Result<()> {
        self.flush();
        Ok(())
    }
}

impl Drop for FdWriter {
    #[inline]
    fn drop(&mut self) {
        self.flush();
    }
}
