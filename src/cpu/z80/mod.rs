#[macro_use]
pub mod inst;

pub mod cpu;
pub mod data;
pub mod regs;

mod error;
pub use self::error::*;
