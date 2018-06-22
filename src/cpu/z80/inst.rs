use std::io;

use byteorder::{ReadBytesExt, LittleEndian};

use super::data::Data;
use super::regs::Registers;

// Context trait defines a context where instructions are executed
pub trait Context {
    fn regs(&self) -> &Registers;
    fn regs_mut(&mut self) -> &mut Registers;
}

// Src defines a source operand of a instruction
pub enum Src<T: Data> {
    Literal(T::Value),
    Register(T::Reg),
}

impl<T: Data> Src<T> {
    pub fn read<C: Context>(&self, c: &C) -> T::Value {
        match self {
            Src::Literal(v) => *v,
            Src::Register(r) => T::read_reg(c.regs(), *r),
        }
    }
}

// Dest defines a destination operand of a instruction
pub enum Dest<T: Data> {
    Register(T::Reg),
}

impl<T: Data> Dest<T> {
    pub fn read<C: Context>(&self, c: &C) -> T::Value {
        match self {
            Dest::Register(r) => T::read_reg(c.regs(), *r),
        }
    }

    pub fn write<C: Context>(&self, c: &mut C, val: T::Value) {
        match self {
            Dest::Register(r) => T::write_reg(c.regs_mut(), *r, val),
        }
    }
}

// Inst trait describes a executable instruction over a context.
pub trait Inst {
    fn exec<C: Context>(&self, ctx: &mut C);
}

pub struct Nop{}

impl Inst for Nop {
    fn exec<C: Context>(&self, ctx: &mut C) {
        ctx.regs_mut().inc_pc(1)
    }
}

pub struct Inc<T: Data>(Dest<T>);

impl<T: Data> Inst for Inc<T> {
    fn exec<C: Context>(&self, ctx: &mut C) {
        let val = self.0.read(ctx);
        self.0.write(ctx, T::inc(val));
        ctx.regs_mut().inc_pc(1)
    }
}

pub struct Load<T: Data>(Dest<T>, Src<T>);

impl<T: Data> Inst for Load<T> {
    fn exec<C: Context>(&self, ctx: &mut C) {
        let val = self.1.read(ctx);
        self.0.write(ctx, val);
        ctx.regs_mut().inc_pc(1)
    }
}

pub trait Decoder {
    fn handle<I: Inst>(&mut self, i: &I);

    fn decode<R: io::Read>(&mut self, input: &mut R) -> io::Result<()> {
        let opcode = input.read_u8()?;
        match opcode {
            0x00 => { self.handle(&Nop{}); Ok({}) },
            _ => unimplemented!("decoding of given opcode is not implemented"),
        }
    }
}