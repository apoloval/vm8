use std::time::{Duration, Instant};
use std::thread;

use bus::Memory16;

use bus;
use cpu::z80::Result;
use cpu::z80::inst::{Context, Inst};
use cpu::z80::regs::Registers;

pub struct CPU<M: Memory16> {
    mem: M,
    regs: Registers,
    clock_period: Duration,
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
        let t0 = Instant::now();
        let inst = self.decode_inst();
        let cycles = inst.exec(self);
        let t1 = t0 + (self.clock_period * cycles as u32);
        let wait = t1 - Instant::now();
        thread::sleep(wait);
        Ok({})
    }

    pub fn decode_inst(&mut self) -> Inst {
        let mut mread = bus::read_from(&self.mem, self.regs.pc());
        Inst::decode(&mut mread).expect("memory read should never fail")
    }
}