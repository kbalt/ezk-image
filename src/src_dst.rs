use crate::{ColorInfo, PixelFormat, Rect};
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
pub struct Source<'a> {
    pub(crate) format: PixelFormat,
    pub(crate) color: ColorInfo,

    pub(crate) buf: &'a [u8],
    pub(crate) width: usize,
    pub(crate) height: usize,

    pub(crate) window: Option<Rect>,
}

impl<'a> Source<'a> {
    pub fn new(
        format: PixelFormat,
        color: ColorInfo,
        buf: &'a [u8],
        width: usize,
        height: usize,
    ) -> Self {
        assert!(format.buffer_size(width, height) <= buf.len());

        Self {
            format,
            color,
            buf,
            width,
            height,
            window: None,
        }
    }

    pub fn with_window(mut self, window: Rect) -> Self {
        self.set_window(window);
        self
    }

    pub fn set_window(&mut self, window: Rect) {
        assert!(window.x + window.width <= self.width);
        assert!(window.y + window.height <= self.height);

        self.window = Some(window)
    }
}

pub struct Dst<'a> {
    pub(crate) format: PixelFormat,
    pub(crate) color: ColorInfo,

    pub(crate) buf: RawMutSliceU8<'a>,
    pub(crate) width: usize,
    pub(crate) height: usize,

    pub(crate) window: Option<Rect>,
}

impl<'a> Dst<'a> {
    pub fn new(
        format: PixelFormat,
        color: ColorInfo,
        buf: &'a mut [u8],
        width: usize,
        height: usize,
    ) -> Self {
        assert!(format.buffer_size(width, height) <= buf.len());

        Self {
            format,
            color,
            buf: RawMutSliceU8::from(buf),
            width,
            height,
            window: None,
        }
    }

    pub fn with_window(mut self, window: Rect) -> Self {
        self.set_window(window);
        self
    }

    pub fn set_window(&mut self, window: Rect) {
        assert!(window.x + window.width <= self.width);
        assert!(window.y + window.height <= self.height);

        self.window = Some(window)
    }

    #[cfg(feature = "multi-thread")]
    pub(crate) fn unsafe_copy_for_multi_threading(&self) -> Self {
        Self {
            format: self.format,
            color: self.color,
            buf: self.buf.unsafe_explicit_copy(),
            width: self.width,
            height: self.height,
            window: self.window,
        }
    }
}

/// Basically `&mut [u8]` which is copyable and shareable across thread without any safety guarantees
///
/// This allows sharing non-contiguous parts of a buffer, but the code must make sure to not write to the same parts
/// of the buffer as other threads, throwing rust safety grantees out of the window.
pub(crate) struct RawMutSliceU8<'a> {
    ptr: *mut u8,
    len: usize,

    _m: PhantomData<&'a mut u8>,
}

impl RawMutSliceU8<'_> {
    #[cfg(feature = "multi-thread")]
    fn unsafe_explicit_copy(&self) -> Self {
        Self {
            ptr: self.ptr,
            len: self.len,
            _m: PhantomData,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn ptr(&self) -> *mut u8 {
        self.ptr
    }
}

unsafe impl Send for RawMutSliceU8<'_> {}
unsafe impl Sync for RawMutSliceU8<'_> {}

impl<'a> From<&'a mut [u8]> for RawMutSliceU8<'a> {
    fn from(slice: &'a mut [u8]) -> Self {
        Self {
            ptr: slice.as_mut_ptr(),
            len: slice.len(),
            _m: PhantomData,
        }
    }
}
