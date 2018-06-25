use bus::Memory16;

use bus;
use cpu::z80::Result;
use cpu::z80::inst::{Context, Inst};
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
    pub fn exec_step(&mut self) -> Result<()> {
        let inst = self.decode_inst();
        inst.exec(self);
        Ok({})
    }

    pub fn decode_inst(&mut self) -> Inst {
        let mut mread = bus::read_from(&self.mem, self.regs.pc());
        Inst::decode(&mut mread).expect("memory read should never fail")
    }
}