use bus::{Address, MemoryItem};
use cpu::z80::data::{Data, Word, Byte};
use cpu::z80::inst::Context;
use cpu::z80::regs::{Reg16, Register};

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
    IndReg(Reg16),
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
    IndReg(Reg16),
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