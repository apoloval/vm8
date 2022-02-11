mod cmd;
mod mmu;
mod system;

pub use cmd::Command;
pub use system::System;

pub enum Addr {
    Logical(u16),
    Physical(u32),
}
