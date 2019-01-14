use crate::bus;
use crate::mem;

// The Z80 memory bus (16-bits addresses with byte values)
pub trait MemoryBus : bus::Bus<Addr=u16, Data=u8> {}
impl<T> MemoryBus for T where T: bus::Bus<Addr=u16, Data=u8> {}

// A memory bank suitable to be used in a Z80 processor.
pub type MemoryBank = mem::MemoryBank<u16>;
