use crate::bus::Bus;

/// The Z80 memory bus (16-bits addresses with byte values)
pub trait Memory : Bus<Addr=u16, Data=u8> {}
impl<T> Memory for T where T: Bus<Addr=u16, Data=u8> {}

/// The Z80 IO bus (16-bits addresses with byte values)
pub trait IO : Bus<Addr=u8, Data=u8> {}
impl<T> IO for T where T: Bus<Addr=u8, Data=u8> {}
