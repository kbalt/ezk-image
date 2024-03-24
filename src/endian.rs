pub(crate) struct BigEndian;
pub(crate) struct LittleEndian;

pub(crate) trait Endian {
    const IS_NATIVE: bool;
}

impl Endian for BigEndian {
    const IS_NATIVE: bool = cfg!(target_endian = "big");
}

impl Endian for LittleEndian {
    const IS_NATIVE: bool = cfg!(target_endian = "little");
}
