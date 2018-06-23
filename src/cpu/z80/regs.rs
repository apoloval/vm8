use super::data::{Data};

pub trait Register<T> {
    fn read(&self, regs: &Registers) -> T;
    fn write(&self, regs: &mut Registers, val: T);
}

#[derive(Clone, Copy)]
pub enum Reg8 { A, B, C, D, E }

impl Register<i8> for Reg8 {
    fn read(&self, regs: &Registers) -> i8 {
        match self {
            Reg8::A => (regs.af >> 8) as i8,
            Reg8::B => (regs.bc >> 8) as i8,
            Reg8::C => (regs.bc) as i8,
            Reg8::D => (regs.de >> 8) as i8,
            Reg8::E => (regs.de) as i8,
        }
    }

    fn write(&self, regs: &mut Registers, val: i8) {
        match self {
            Reg8::A => regs.af = (regs.af & 0x00ff) | ((val as u16) << 8),
            Reg8::B => regs.bc = (regs.bc & 0x00ff) | ((val as u16) << 8),
            Reg8::C => regs.bc = (regs.bc & 0xff00) | (val as u16),
            Reg8::D => regs.de = (regs.de & 0x00ff) | ((val as u16) << 8),
            Reg8::E => regs.de = (regs.de & 0xff00) | (val as u16),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Reg16 { AF, BC, DE }

impl Register<i16> for Reg16 {
    fn read(&self, regs: &Registers) -> i16 {
        match self {
            Reg16::AF => regs.af as i16,
            Reg16::BC => regs.bc as i16,
            Reg16::DE => regs.de as i16,
        }
    }

    fn write(&self, regs: &mut Registers, val: i16) {
        match self {
            Reg16::AF => regs.af = val as u16,
            Reg16::BC => regs.bc = val as u16,
            Reg16::DE => regs.de = val as u16,
        }
    }
}

pub struct Registers {
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub pc: u16,
}

impl Registers {
    pub fn read<T: Data>(&self, reg: T::Reg) -> T::Value {
        T::read_reg(self, reg)
    }

    pub fn write<T: Data>(&mut self, reg: T::Reg, val: T::Value) {
        T::write_reg(self, reg, val)
    }

    pub fn inc_pc(&mut self, val: u16) {
        self.pc += val
    }
}