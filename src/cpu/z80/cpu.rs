use bus::Memory16;

use cpu::z80::Result;
use cpu::z80::inst::Context;
use cpu::z80::regs::Registers;

pub struct CPU<M: Memory16> {
    mem: M,
    regs: Registers,
}

impl<M: Memory16> Context for CPU<M> {
    type Mem = M;
    fn regs(&self) -> &Registers { &self.regs }
    fn regs_mut(&mut self) -> &mut Registers { &mut self.regs }
    fn mem(&self) -> &M { &self.mem }
    fn mem_mut(&mut self) -> &mut M { &mut self.mem }
}

impl<M: Memory16> CPU<M> {
    fn exec_step(&mut self) -> Result<()> {
        unimplemented!()
    }
}