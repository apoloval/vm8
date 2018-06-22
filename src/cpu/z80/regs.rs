use super::data::{Data};

#[derive(Clone, Copy)]
pub enum Reg8 { A, B, C, D, E }

#[derive(Clone, Copy)]
pub enum Reg16 { AF, BC, DE }

pub struct Registers {
    pub af: i16,
    pub bc: i16,
    pub de: i16,
}

impl Registers {
    pub fn read<T: Data>(&self, reg: T::Reg) -> T::Value {
        T::read_reg(self, reg)
    }

    pub fn write<T: Data>(&mut self, reg: T::Reg, val: T::Value) {
        T::write_reg(self, reg, val)
    }
}