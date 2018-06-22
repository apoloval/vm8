use std::ops::Add;

use byteorder::LittleEndian;

use bus::Memory;
use cpu::z80::regs::{Reg8, Reg16, Registers};

pub trait Data {
    type Reg: Copy;
    type Value: Copy + Add<Output=Self::Value>;

    fn unit() -> Self::Value;
    fn read_reg(regs: &Registers, reg: Self::Reg) -> Self::Value;
    fn write_reg(regs: &mut Registers, reg: Self::Reg, val: Self::Value);
    fn read_mem<M: Memory<Addr=u16>>(mem: &M, addr: u16) -> Self::Value;

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

    fn read_mem<M: Memory<Addr=u16>>(mem: &M, addr: u16) -> i8 {
        mem.read_i8(addr)
    }

    fn write_reg(regs: &mut Registers, reg: Reg8, val: i8) {
        match reg {
            Reg8::A => regs.af = (regs.af & 0x00ff) | ((val as i16) << 8),
            Reg8::B => regs.bc = (regs.bc & 0x00ff) | ((val as i16) << 8),
            Reg8::C => regs.bc = (regs.bc & 0xff00) | (val as i16),
            Reg8::D => regs.de = (regs.de & 0x00ff) | ((val as i16) << 8),
            Reg8::E => regs.de = (regs.de & 0xff00) | (val as i16),
        }
    }
}

pub struct Word;

impl Data for Word {
    type Reg = Reg16;
    type Value = i16;

    fn unit() -> i16 { return 1 }

    fn read_reg(regs: &Registers, reg: Reg16) -> i16 {
        match reg {
            Reg16::AF => regs.af,
            Reg16::BC => regs.bc,
            Reg16::DE => regs.de,
        }
    }

    fn read_mem<M: Memory<Addr=u16>>(mem: &M, addr: u16) -> i16 {
        mem.read_i16::<LittleEndian>(addr)
    }

    fn write_reg(regs: &mut Registers, reg: Reg16, val: i16) {
        match reg {
            Reg16::AF => regs.af = val,
            Reg16::BC => regs.bc = val,
            Reg16::DE => regs.de = val,
        }
    }
}
