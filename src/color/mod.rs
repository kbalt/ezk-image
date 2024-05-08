pub(crate) mod primaries;
pub(crate) mod space;
pub(crate) mod transfer;

pub use primaries::ColorPrimaries;
pub use space::ColorSpace;
pub use transfer::ColorTransfer;

/// Color space of an image
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorInfo {
    /// Describes the YUV color space, not required for RGB images
    pub space: ColorSpace,
    pub transfer: ColorTransfer,
    pub primaries: ColorPrimaries,

    /// For YUV images
    ///
    /// If the image uses either full or standard range
    ///
    /// - full range (0 - 255)
    /// - standard range Y (16 - 235), U & V (16 - 240)
    pub full_range: bool,
}

pub(crate) mod mat_idxs {
    pub(crate) const Y: usize = 0;
    pub(crate) const U: usize = 1;
    pub(crate) const V: usize = 2;

    pub(crate) const R: usize = 0;
    pub(crate) const G: usize = 1;
    pub(crate) const B: usize = 2;
}

pub(crate) struct ColorOps {
    pub(crate) rgb_to_xyz: &'static [[f32; 3]; 3],
    pub(crate) xyz_to_rgb: &'static [[f32; 3]; 3],

    pub(crate) space: ColorSpace,
    pub(crate) transfer: ColorTransfer,
}

impl ColorOps {
    pub(crate) fn from_info(info: &ColorInfo) -> Self {
        let rgb_to_xyz = info.primaries.rgb_to_xyz_mat();
        let xyz_to_rgb = info.primaries.xyz_to_rgb_mat();

        Self {
            rgb_to_xyz,
            xyz_to_rgb,

            space: info.space,
            transfer: info.transfer,
        }
    }
}
