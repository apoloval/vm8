use super::inst::{Context, Decoder, Inst};
use super::regs::Registers;

pub struct CPU {
    regs: Registers,
}

impl Context for CPU {
    fn regs(&self) -> &Registers { &self.regs }
    fn regs_mut(&mut self) -> &mut Registers { &mut self.regs }
}

impl Decoder for CPU {
    fn handle<I: Inst>(&mut self, inst: &I) {
        inst.exec(self)
    }
}
