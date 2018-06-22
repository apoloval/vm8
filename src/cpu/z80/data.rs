use std::ops::Add;

use super::regs::{Reg8, Reg16, Registers};

pub trait Data {
    type Reg: Copy;
    type Value: Copy + Add<Output=Self::Value>;

    fn unit() -> Self::Value;
    fn read_reg(regs: &Registers, reg: Self::Reg) -> Self::Value;
    fn write_reg(regs: &mut Registers, reg: Self::Reg, val: Self::Value);

    fn inc(v: Self::Value) -> Self::Value { v + Self::unit() }
}

pub struct Byte;

impl Data for Byte {
    type Reg = Reg8;
    type Value = i8;

    fn unit() -> i8 { return 1 }

    fn read_reg(regs: &Registers, reg: Reg8) -> i8 {
        match reg {
            Reg8::A => (regs.af >> 8) as i8,
            Reg8::B => (regs.bc >> 8) as i8,
            Reg8::C => (regs.bc) as i8,
            Reg8::D => (regs.de >> 8) as i8,
            Reg8::E => (regs.de) as i8,
        }
    }

    fn write_reg(regs: &mut Registers, reg: Reg8, val: i8) {
        unimplemented!()
    }
}

pub struct Word;

impl Data for Word {
    type Reg = Reg16;
    type Value = i16;

    fn unit() -> i16 { return 1 }

    fn read_reg(regs: &Registers, reg: Reg16) -> i16 {
        unimplemented!()
    }

    fn write_reg(regs: &mut Registers, reg: Reg16, val: i16) {
        unimplemented!()
    }
}
