use bus::Memory;
use cpu::z80::reg::Registers;

#[macro_use]
mod defs;

mod ops;
use self::ops::*;

mod dec;
pub use self::dec::Decoder;

mod exec;
use self::exec::execute;

// Context trait defines a context where instructions are executed
pub trait Context {
    type Mem: Memory;
    fn regs(&self) -> &Registers;
    fn regs_mut(&mut self) -> &mut Registers;
    fn mem(&self) -> &Self::Mem;
    fn mem_mut(&mut self) -> &mut Self::Mem;
}

#[derive(Debug, Eq, PartialEq)]
pub enum Mnemo { ADD, DEC, EX, INC, JP, LD, NOP, RLCA, RRCA }

#[derive(Debug, Eq, PartialEq)]
pub enum Operands {
    Nulary,
    UnaryDest8(Dest8),
    UnaryDest16(Dest16),
    UnarySrc8(Src8),
    UnarySrc16(Src16),
    Binary8(Dest8, Src8),    
    Binary16(Dest16, Src16),
}

pub type OpCode = u32;
pub type Size = usize;
pub type Cycles = usize;

#[derive(Debug, Eq, PartialEq)]
pub struct Inst {
    opcode: OpCode,
    mnemo: Mnemo,
    ops: Operands,
    size: Size,
    cycles: Cycles,
}

impl Inst {    
    pub fn exec<C: Context>(&self, ctx: &mut C) -> Cycles {
        execute(self, ctx)
    }
}
