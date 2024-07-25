mod read;
mod write;

pub(crate) use read::PackedYuv422Reader;
pub(crate) use write::PackedYuv422Writer;

#[repr(u8)]
pub(crate) enum PackedYuv422Format {
    YUYV = 0,
    UYVY = 1,
}
