macro_rules! platform_trait {
    ($trait:ident:$trait_impl:ident) => {
        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86", target_arch = "x86_64")))]
        pub(crate) trait $trait: $trait_impl<f32> {}
        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86", target_arch = "x86_64")))]
        impl<T: $trait_impl<f32>> $trait for T {}

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        pub(crate) trait $trait:
            $trait_impl<f32> + $trait_impl<$crate::arch::__m256>
        {
        }
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        impl<T: $trait_impl<f32> + $trait_impl<$crate::arch::__m256>> $trait for T {}

        #[cfg(target_arch = "aarch64")]
        pub(crate) trait $trait:
            $trait_impl<f32> + $trait_impl<$crate::arch::float32x4_t>
        {
        }
        #[cfg(target_arch = "aarch64")]
        impl<T: $trait_impl<f32> + $trait_impl<$crate::arch::float32x4_t>> $trait for T {}
    };
}

mod i420;
mod i420_read_u16;
mod i420_read_u8;
mod i420_to_rgb;
mod i420_u16_write;
mod i420_u8_write;
mod rgb;
mod rgb_read;
mod rgb_to_i420;
mod rgb_transfer_and_primaries_convert;
mod rgb_write;
mod rgba;
mod rgba_read;

pub(crate) use self::{
    i420_read_u16::read_i420_u16,
    i420_read_u8::read_i420,
    i420_to_rgb::I420ToRgbVisitor,
    i420_u16_write::I420U16Writer,
    i420_u8_write::I420U8Writer,
    rgb::RgbBlockVisitor,
    rgb_read::read_rgb_4x,
    rgb_to_i420::RgbToI420Visitor,
    rgb_transfer_and_primaries_convert::RgbTransferAndPrimariesConvert,
    rgb_write::{RGBAWriter, RGBWriter},
    rgba_read::read_rgba_4x,
};
