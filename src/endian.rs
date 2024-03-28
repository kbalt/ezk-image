pub struct NativeEndian;
pub struct BigEndian;
pub struct LittleEndian;

pub trait Endian {
    const IS_NATIVE: bool;
}

impl Endian for NativeEndian {
    const IS_NATIVE: bool = true;
}

impl Endian for BigEndian {
    const IS_NATIVE: bool = cfg!(target_endian = "big");
}

impl Endian for LittleEndian {
    const IS_NATIVE: bool = cfg!(target_endian = "little");
}
