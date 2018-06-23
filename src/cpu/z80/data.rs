use std::ops::Add;

use byteorder::LittleEndian;

use bus::Memory;
use cpu::z80::regs::{Reg8, Reg16, Register, Registers};

pub trait Data {
    type Reg: Copy + Register<Self::Value>;
    type Value: Copy + Add<Output=Self::Value>;

    fn unit() -> Self::Value;
    fn read_reg(regs: &Registers, reg: Self::Reg) -> Self::Value;
    fn write_reg(regs: &mut Registers, reg: Self::Reg, val: Self::Value);
    fn read_mem<M: Memory<Addr=u16>>(mem: &M, addr: u16) -> Self::Value;
    fn write_mem<M: Memory<Addr=u16>>(mem: &mut M, addr: u16, val: Self::Value);

    fn inc(v: Self::Value) -> Self::Value { v + Self::unit() }
}

pub struct Byte;

impl Data for Byte {
    type Reg = Reg8;
    type Value = i8;

    fn unit() -> i8 { return 1 }

    fn read_reg(regs: &Registers, reg: Reg8) -> i8 {
        reg.read(regs)    
    }

    fn read_mem<M: Memory<Addr=u16>>(mem: &M, addr: u16) -> i8 {
        mem.read_i8(addr)
    }

    fn write_reg(regs: &mut Registers, reg: Reg8, val: i8) {
        reg.write(regs, val)
    }

    fn write_mem<M: Memory<Addr=u16>>(mem: &mut M, addr: u16, val: i8) {
        mem.write_i8(addr, val)
    }
}

pub struct Word;

impl Data for Word {
    type Reg = Reg16;
    type Value = i16;

    fn unit() -> i16 { return 1 }

    fn read_reg(regs: &Registers, reg: Reg16) -> i16 {
        reg.read(regs)
    }

    fn read_mem<M: Memory<Addr=u16>>(mem: &M, addr: u16) -> i16 {
        mem.read_i16::<LittleEndian>(addr)
    }

    fn write_reg(regs: &mut Registers, reg: Reg16, val: i16) {
        reg.write(regs, val)
    }

    fn write_mem<M: Memory<Addr=u16>>(mem: &mut M, addr: u16, val: i16) {
        mem.write_i16::<LittleEndian>(addr, val)
    }
}
