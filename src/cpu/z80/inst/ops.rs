use byteorder::{ByteOrder, LittleEndian};
use num_traits::{Num, One};

use bus::{Address, MemoryItem};
use cpu::z80::inst::Context;
use cpu::z80::reg;
use cpu::z80::reg::{Read, Write};

pub trait Data {
    type Ord: ByteOrder;
    type Value: Num + MemoryItem<Self::Ord> + Copy;
    type Reg: reg::Read<Self::Value> + reg::Write<Self::Value>;

    fn inc(v: Self::Value) -> Self::Value { v + Self::Value::one() }
    fn dec(v: Self::Value) -> Self::Value { v - Self::Value::one() }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Byte;

impl Data for Byte {
    type Ord = LittleEndian;
    type Value = u8;
    type Reg = reg::Name8;
}

#[derive(Debug, Eq, PartialEq)]
pub struct Word;

impl Data for Word {
    type Ord = LittleEndian;
    type Value = u16;
    type Reg = reg::Name16;
}

pub trait OpRead<T> {
    fn read<C: Context>(&self, c: &C) -> T;
}

pub trait OpWrite<T> {
    fn write<C: Context>(&self, c: &mut C, val: T);
}

// Src defines a source operand of a instruction
#[derive(Debug, Eq, PartialEq)]
pub enum Src<D: Data> {
    Liter(D::Value),
    Reg(D::Reg),
    IndReg(reg::Name16),
}

pub type Src8 = Src<Byte>;
pub type Src16 = Src<Word>;

impl<D: Data> Src<D> {
    pub fn read<C: Context>(&self, c: &C) -> D::Value {
        match self {
            Src::Liter(v) => *v,
            Src::Reg(r) => r.read(c.regs()),
            Src::IndReg(r) => {
                let addr = Address::from(r.read(c.regs()) as u16);
                D::Value::mem_read(c.mem(), addr)
            },
        }
    }
}

// Dest defines a destination operand of a instruction
#[derive(Debug, Eq, PartialEq)]
pub enum Dest<D: Data> {
    Reg(D::Reg),
    IndReg(reg::Name16),
}

impl<D: Data> Dest<D> {
    pub fn read<C: Context>(&self, c: &C) -> D::Value {
        match self {
            Dest::Reg(r) => r.read(c.regs()),
            Dest::IndReg(r) => {
                let addr = Address::from(r.read(c.regs()) as u16);
                D::Value::mem_read(c.mem(), addr)
            },
        }
    }

    pub fn write<C: Context>(&self, c: &mut C, val: D::Value) {
        match self {
            Dest::Reg(r) => r.write(c.regs_mut(), val),
            Dest::IndReg(r) => {
                let addr = Address::from(r.read(c.regs()) as u16);
                D::Value::mem_write(c.mem_mut(), addr, val)
            },
        }
    }
}

pub type Dest8 = Dest<Byte>;
pub type Dest16 = Dest<Word>;
