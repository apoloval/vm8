#[macro_use] pub mod inst;
#[macro_use] pub mod eval;
#[macro_use] pub mod flags;

#[cfg(test)]
#[macro_use]
mod assert;

mod alu;
mod device;
mod exec;
mod mem;
mod reg;

pub use self::device::{CPU, Options};
pub use self::mem::MemoryBank;
