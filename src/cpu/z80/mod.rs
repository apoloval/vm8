use bus;
use mem;

#[macro_use] pub mod inst;
#[macro_use] pub mod flags;
#[macro_use] mod cpu;

mod alu;
mod error;
mod exec;
mod reg;

pub use self::cpu::*;
pub use self::inst::*;
pub use self::error::*;
pub use self::exec::*;
pub use self::reg::*;

// The Z80 memory bus (16-bits addresses with byte values)
pub trait MemoryBus : bus::Bus<Addr=u16, Data=u8> {}
impl<T> MemoryBus for T where T: bus::Bus<Addr=u16, Data=u8> {}

// A memory bank suitable to be used in a Z80 processor.
pub type MemoryBank = mem::MemoryBank<u16>;