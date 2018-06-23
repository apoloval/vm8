use byteorder::LittleEndian;
use num_traits::{Num, One};

use bus::Memory;
use cpu::z80::regs::{Reg8, Reg16, Register};

pub trait Data {
    type Reg: Register<Self::Value>;
    type Value: Num + Copy;

    fn read_mem<M: Memory<Addr=u16>>(mem: &M, addr: u16) -> Self::Value;
    fn write_mem<M: Memory<Addr=u16>>(mem: &mut M, addr: u16, val: Self::Value);

    fn inc(v: Self::Value) -> Self::Value { v + Self::Value::one() }
}

pub struct Byte;

impl Data for Byte {
    type Reg = Reg8;
    type Value = i8;

    fn read_mem<M: Memory<Addr=u16>>(mem: &M, addr: u16) -> i8 {
        mem.read_i8(addr)
    }

    fn write_mem<M: Memory<Addr=u16>>(mem: &mut M, addr: u16, val: i8) {
        mem.write_i8(addr, val)
    }
}

pub struct Word;

impl Data for Word {
    type Reg = Reg16;
    type Value = i16;

    fn read_mem<M: Memory<Addr=u16>>(mem: &M, addr: u16) -> i16 {
        mem.read_i16::<LittleEndian>(addr)
    }

    fn write_mem<M: Memory<Addr=u16>>(mem: &mut M, addr: u16, val: i16) {
        mem.write_i16::<LittleEndian>(addr, val)
    }
}
