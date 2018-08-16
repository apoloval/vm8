use bus::Memory;
use cpu::z80::reg::Registers;

#[macro_use]
mod defs;

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

pub type OpCode = u32;
pub type Size = usize;
pub type Cycles = usize;

#[derive(Debug, Eq, PartialEq)]
pub struct Inst {
    opcode: OpCode,
    extra8: u8,
    extra16: u16,
    size: Size,
    cycles: Cycles,
}

impl Inst {    
    pub fn exec<C: Context>(&self, ctx: &mut C) -> Cycles {
        execute(self, ctx)
    }
}
