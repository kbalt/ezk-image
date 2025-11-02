mod dyn_rgba_src;
mod transfer_and_primaries_convert;
mod visit_2x2;

#[cfg(any(
    feature = "I420",
    feature = "I010",
    feature = "I012",
    feature = "NV12",
    feature = "P010",
    feature = "P012"
))]
pub(crate) mod yuv420;
#[cfg(any(feature = "I422", feature = "I210", feature = "I212", feature = "YUYV"))]
pub(crate) mod yuv422;
#[cfg(any(feature = "I444", feature = "I410", feature = "I412"))]
pub(crate) mod yuv444;

pub(crate) mod rgb;

pub(crate) use transfer_and_primaries_convert::{
    TransferAndPrimariesConvert, need_transfer_and_primaries_convert,
};

pub(crate) use dyn_rgba_src::{DynRgbaReader, DynRgbaReaderSpec};

fn max_value_for_bits(bits: usize) -> f32 {
    ((1 << bits) - 1) as f32
}
