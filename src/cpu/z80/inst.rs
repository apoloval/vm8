use std::io;

use byteorder::{ReadBytesExt, LittleEndian};

use super::data::{Data, Word, Byte};
use super::regs::{Reg8, Reg16, Registers};

// Context trait defines a context where instructions are executed
pub trait Context {
    fn regs(&self) -> &Registers;
    fn regs_mut(&mut self) -> &mut Registers;
}

// Src defines a source operand of a instruction
pub enum Src<T: Data> {
    Liter(T::Value),
    Reg(T::Reg),
}

impl<T: Data> Src<T> {
    pub fn read<C: Context>(&self, c: &C) -> T::Value {
        match self {
            Src::Liter(v) => *v,
            Src::Reg(r) => T::read_reg(c.regs(), *r),
        }
    }
}

// Dest defines a destination operand of a instruction
pub enum Dest<T: Data> {
    Reg(T::Reg),
    IndReg(Reg16),
}

impl<T: Data> Dest<T> {
    pub fn read<C: Context>(&self, c: &C) -> T::Value {
        match self {
            Dest::Reg(r) => T::read_reg(c.regs(), *r),
            Dest::IndReg(r) => {
                let addr = Word::read_reg(c.regs(), *r);
                unimplemented!()
            },
        }
    }

    pub fn write<C: Context>(&self, c: &mut C, val: T::Value) {
        match self {
            Dest::Reg(r) => T::write_reg(c.regs_mut(), *r, val),
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
            0x00 => self.handle(&Nop{}),
            0x01 => self.handle(&Load::<Word>(
                Dest::Reg(Reg16::BC), 
                Src::Liter(input.read_i16::<LittleEndian>()?),
            )),
            _ => unimplemented!("decoding of given opcode is not implemented"),
        };
        Ok({})
    }
}