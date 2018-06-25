use std::io;

use byteorder::{ReadBytesExt, LittleEndian};

use bus::{Addr16, Memory16, MemoryItem};
use cpu::z80::data::{Data, Word, Byte};
use cpu::z80::regs::{Reg8, Reg16, Register, Registers};

// Context trait defines a context where instructions are executed
pub trait Context {
    type Mem: Memory16;
    fn regs(&self) -> &Registers;
    fn regs_mut(&mut self) -> &mut Registers;
    fn mem(&self) -> &Self::Mem;
    fn mem_mut(&mut self) -> &mut Self::Mem;
}

pub trait OpRead<T> {
    fn read<C: Context>(&self, c: &C) -> T;
}

pub trait OpWrite<T> {
    fn write<C: Context>(&self, c: &mut C, val: T);
}

// Src defines a source operand of a instruction
#[derive(Debug, PartialEq)]
pub enum Src<D: Data> {
    Liter(D::Value),
    Reg(D::Reg),
}

type Src8 = Src<Byte>;
type Src16 = Src<Word>;

impl<D: Data> Src<D> {
    fn read<C: Context>(&self, c: &C) -> D::Value {
        match self {
            Src::Liter(v) => *v,
            Src::Reg(r) => r.read(c.regs()),
        }
    }
}

// Dest defines a destination operand of a instruction
#[derive(Debug, PartialEq)]
pub enum Dest<D: Data> {
    Reg(D::Reg),
    IndReg(Reg16),
}

impl<D: Data> Dest<D> {
    fn read<C: Context>(&self, c: &C) -> D::Value {
        match self {
            Dest::Reg(r) => r.read(c.regs()),
            Dest::IndReg(r) => {
                let addr = Addr16::from(r.read(c.regs()) as u16);
                D::Value::mem_read(c.mem(), addr)
            },
        }
    }

    fn write<C: Context>(&self, c: &mut C, val: D::Value) {
        match self {
            Dest::Reg(r) => r.write(c.regs_mut(), val),
            Dest::IndReg(r) => {
                let addr = Addr16::from(r.read(c.regs()) as u16);
                D::Value::mem_write(c.mem_mut(), addr, val)
            },
        }
    }
}

type Dest8 = Dest<Byte>;
type Dest16 = Dest<Word>;

type InstSize = usize;
type InstTime = usize;

#[derive(Debug, PartialEq)]
pub struct Inst {
    action: Action,
    size: InstSize,
    time: InstTime,
}

impl Inst {
    pub fn exec<C: Context>(&self, ctx: &mut C) -> InstTime {
        self.action.exec(ctx, self.size);
        self.time
    }

    pub fn decode<R: io::Read>(input: &mut R) -> io::Result<Inst> {
        let opcode = input.read_u8()?;
        let inst = match opcode {
            0x00 => Inst {
                action: Action::Nop, 
                size: 1, 
                time: 4,
            },
            0x01 => Inst {
                action: Action::Load16(
                    Dest::Reg(Reg16::BC), 
                    Src::Liter(input.read_u16::<LittleEndian>()?),
                ), 
                size: 3,
                time: 10,
            },
            0x02 => Inst {
                action: Action::Load8(
                    Dest::IndReg(Reg16::BC), 
                    Src::Reg(Reg8::A),
                ), 
                size: 1,
                time: 7,
            },
            _ => unimplemented!("decoding of given opcode is not implemented"),
        };
        Ok(inst)
    }
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Nop,
    Inc8(Dest8),
    Load8(Dest8, Src8),
    Load16(Dest16, Src16),
}

impl Action {
    pub fn exec<C: Context>(&self, ctx: &mut C, size: InstSize) {
        match self {
            Action::Nop => Self::exec_nop(ctx, size),
            Action::Inc8(dst) => Self::exec_inc(ctx, size, dst),
            Action::Load8(dst, src) => Self::exec_load(ctx, size, dst, src),
            Action::Load16(dst, src) => Self::exec_load(ctx, size, dst, src),
        }
    }

    fn exec_nop<C: Context>(ctx: &mut C, size: InstSize) {
        ctx.regs_mut().inc_pc(size)
    }

    fn exec_inc<C: Context, D: Data>(ctx: &mut C, size: InstSize, dst: &Dest<D>) {
        let val = dst.read(ctx);
        dst.write(ctx, D::inc(val));
        ctx.regs_mut().inc_pc(size)
    }

    fn exec_load<C: Context, D: Data>(ctx: &mut C, size: InstSize, dst: &Dest<D>, src: &Src<D>) {
        let val = src.read(ctx);
        dst.write(ctx, val);
        ctx.regs_mut().inc_pc(size)
    }
}

#[cfg(test)]
mod test {
    use cpu::z80::regs::{Reg16};
    use super::*;

    #[test]
    fn encode_nop() {
        test_encode(
            vec![0x00],
            Inst { 
                action: Action::Nop, 
                size: 1, 
                time: 4,
            },
        );
    }
        
    #[test]
    fn encode_load_bc_liter() {
        test_encode(
            vec![0x01, 0x34, 0x12],
            Inst { 
                action: Action::Load16(
                    Dest::Reg(Reg16::BC), 
                    Src::Liter(0x1234),
                ), 
                size: 3, 
                time: 10,
            },
        );
    }
        
    #[test]
    fn encode_load_ind_bc_a() {
        test_encode(
            vec![0x02],
            Inst { 
                action: Action::Load8(
                    Dest::IndReg(Reg16::BC), 
                    Src::Reg(Reg8::A),
                ), 
                size: 1, 
                time: 7,
            },
        );
    }

    fn test_encode(input: Vec<u8>, expected: Inst) {
        let mut read: &[u8] = &input;
        let given = Inst::decode(&mut read).unwrap();
        assert_eq!(expected, given);
    }
}