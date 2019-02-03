use crate::bus::Bus;

/// The Z80 memory bus (16-bits addresses with byte values)
pub trait Memory : Bus<u16, u8> {}
impl<T> Memory for T where T: Bus<u16, u8> {}

/// The Z80 IO bus (16-bits addresses with byte values)
pub trait IO : Bus<u8, u8> {}
impl<T> IO for T where T: Bus<u8, u8> {}
