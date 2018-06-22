use super::inst::{Context, Inst};
use super::regs::Registers;

pub struct CPU {
    regs: Registers,
}

impl Context for CPU {
    fn regs(&self) -> &Registers { &self.regs }
    fn regs_mut(&mut self) -> &mut Registers { &mut self.regs }
}

impl CPU {
    pub fn exec<I: Inst>(&mut self, inst: I) {
        inst.exec(self)
    }
}