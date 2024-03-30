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
mod nv12;
mod rgb;
mod rgba;
mod transfer_and_primaries_convert;

pub(crate) use i420::*;
pub(crate) use nv12::*;
pub(crate) use rgb::*;
pub(crate) use rgba::*;
pub(crate) use transfer_and_primaries_convert::TransferAndPrimariesConvert;
