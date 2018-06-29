#[macro_use]
pub mod inst;

pub mod data;
pub mod regs;

mod cpu;
pub use self::cpu::*;

mod error;
pub use self::error::*;
