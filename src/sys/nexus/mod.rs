use std::ops::Add;

mod cmd;
mod bus;
mod mmu;
mod system;

pub use cmd::Command;
pub use system::System;

#[derive(Clone, Copy)]
pub enum Addr {
    Logical(u16),
    Physical(u32),
}

impl Add<usize> for Addr {
    type Output = Self;
    fn add(self, rhs: usize) -> Self {
        match self {
            Addr::Logical(a) => Addr::Logical(a.wrapping_add(rhs as u16)),
            Addr::Physical(a) => Addr::Physical(a.wrapping_add(rhs as u32)),
        }
    }
}
