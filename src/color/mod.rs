use self::transfer::ColorTransferImpl;
use crate::arch::*;
use crate::vector::Vector;

pub(crate) mod primaries;
pub(crate) mod space;
pub(crate) mod transfer;

pub use primaries::ColorPrimaries;
pub use space::ColorSpace;
pub use transfer::ColorTransfer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorInfo {
    pub space: ColorSpace,
    pub transfer: ColorTransfer,
    pub primaries: ColorPrimaries,
    pub full_range: bool,
}

pub mod mat_idxs {
    pub const Y: usize = 0;
    pub const U: usize = 1;
    pub const V: usize = 2;

    pub const R: usize = 0;
    pub const G: usize = 1;
    pub const B: usize = 2;
}

pub(crate) struct ColorOps {
    pub(crate) rgb_to_xyz: &'static [[f32; 3]; 3],
    pub(crate) xyz_to_rgb: &'static [[f32; 3]; 3],

    pub(crate) space: ColorSpace,

    pub(crate) f32: ColorOpsPart<f32>,
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub(crate) avx2: ColorOpsPart<__m256>,
    #[cfg(target_arch = "aarch64")]
    pub(crate) neon: ColorOpsPart<float32x4_t>,
}

pub(crate) struct ColorOpsPart<V: Vector> {
    pub(crate) transfer: &'static dyn ColorTransferImpl<V>,
}

impl ColorOps {
    pub(crate) fn from_info(info: &ColorInfo) -> Self {
        let rgb_to_xyz = info.primaries.rgb_to_xyz_mat();
        let xyz_to_rgb = info.primaries.xyz_to_rgb_mat();

        let f32 = {
            ColorOpsPart {
                transfer: info.transfer.dispatch(),
            }
        };

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        let avx2 = {
            ColorOpsPart {
                transfer: info.transfer.dispatch(),
            }
        };

        #[cfg(target_arch = "aarch64")]
        let neon = {
            ColorOpsPart {
                transfer: info.transfer.dispatch(),
            }
        };

        Self {
            rgb_to_xyz,
            xyz_to_rgb,

            space: info.space,

            f32,
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            avx2,
            #[cfg(target_arch = "aarch64")]
            neon,
        }
    }
}
