use bus::Memory;

use cpu::z80::inst::{Context, Decoder, Inst};
use cpu::z80::regs::Registers;

pub struct CPU<M: Memory<Addr=u16>> {
    mem: M,
    regs: Registers,
}

impl<M: Memory<Addr=u16>> Context for CPU<M> {
    type Mem = M;
    fn regs(&self) -> &Registers { &self.regs }
    fn regs_mut(&mut self) -> &mut Registers { &mut self.regs }
    fn mem(&self) -> &M { &self.mem }
    fn mem_mut(&mut self) -> &mut M { &mut self.mem }
}

impl<M: Memory<Addr=u16>> Decoder for CPU<M> {
    fn handle<I: Inst>(&mut self, inst: &I) {
        inst.exec(self)
    }
}
