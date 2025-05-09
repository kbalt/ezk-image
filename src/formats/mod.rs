#[macro_use]
mod rgba;
mod dyn_rgba_src;
mod i420;
mod i422;
mod i444;
mod nv12;
mod rgb;
mod transfer_and_primaries_convert;
mod visit_2x2;
mod yuyv;

pub(crate) use dyn_rgba_src::{DynRgbaReader, DynRgbaReaderSpec};
pub(crate) use i420::*;
pub(crate) use i422::*;
pub(crate) use i444::*;
pub(crate) use nv12::*;
pub(crate) use rgb::*;
pub(crate) use rgba::*;
pub(crate) use transfer_and_primaries_convert::{
    TransferAndPrimariesConvert, need_transfer_and_primaries_convert,
};
pub(crate) use yuyv::*;

fn max_value_for_bits(bits: usize) -> f32 {
    ((1 << bits) - 1) as f32
}
