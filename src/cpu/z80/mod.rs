#[macro_use]
pub mod inst;

pub mod reg;

mod cpu;
pub use self::cpu::*;

mod error;
pub use self::error::*;
