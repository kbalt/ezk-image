pub(crate) mod primaries;
pub(crate) mod space;
pub(crate) mod transfer;

pub use primaries::ColorPrimaries;
pub use space::ColorSpace;
pub use transfer::ColorTransfer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColorInfo {
    pub transfer: ColorTransfer,
    pub primaries: ColorPrimaries,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YuvColorInfo {
    pub transfer: ColorTransfer,
    pub primaries: ColorPrimaries,
    pub space: ColorSpace,
    /// If the image uses either full or standard range
    ///
    /// - full range (0 - 255)
    /// - standard range Y (16 - 235), U & V (16 - 240)
    pub full_range: bool,
}

/// Color space of an image
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorInfo {
    RGB(RgbColorInfo),
    YUV(YuvColorInfo),
}

impl ColorInfo {
    pub fn primaries(&self) -> ColorPrimaries {
        match self {
            ColorInfo::RGB(rgb) => rgb.primaries,
            ColorInfo::YUV(yuv) => yuv.primaries,
        }
    }

    pub fn transfer(&self) -> ColorTransfer {
        match self {
            ColorInfo::RGB(rgb) => rgb.transfer,
            ColorInfo::YUV(yuv) => yuv.transfer,
        }
    }
}

impl From<RgbColorInfo> for ColorInfo {
    fn from(value: RgbColorInfo) -> Self {
        ColorInfo::RGB(value)
    }
}

impl From<YuvColorInfo> for ColorInfo {
    fn from(value: YuvColorInfo) -> Self {
        ColorInfo::YUV(value)
    }
}

pub(crate) mod mat_idxs {
    pub(crate) const Y: usize = 0;
    pub(crate) const U: usize = 1;
    pub(crate) const V: usize = 2;

    pub(crate) const R: usize = 0;
    pub(crate) const G: usize = 1;
    pub(crate) const B: usize = 2;
}
