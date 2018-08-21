use cpu::{ExecutionPlan, ExecutionResult, Processor};
use cpu::z80::{Context, MemoryBus, Registers, decode, execute};

pub struct CPU<M: MemoryBus> {
    mem: M,
    regs: Registers,
}

impl<M: MemoryBus> Context for CPU<M> {
    type Mem = M;
    fn regs(&self) -> &Registers { &self.regs }
    fn regs_mut(&mut self) -> &mut Registers { &mut self.regs }
    fn mem(&self) -> &M { &self.mem }
    fn mem_mut(&mut self) -> &mut M { &mut self.mem }
}

impl<M: MemoryBus> Processor for CPU<M> {
    fn execute(&mut self, plan: &ExecutionPlan) -> ExecutionResult {
        let mut result = ExecutionResult::default();
        while !plan.is_completed(&result) {
            let inst = decode(&self.mem, *self.regs.pc);
            result.total_cycles += execute(&inst, self);
            result.total_instructions += 1;
        }
        result
    }
}

impl<M: MemoryBus> CPU<M> {
    pub fn new(mem: M) -> Self {
        Self {
            mem: mem,
            regs: Registers::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use cpu::z80;

    use super::*;

    #[test]
    fn exec_nop() {
        let mut cpu = sample_cpu(&[0x00]);
        let plan = ExecutionPlan::with_max_instructions(10000);
        cpu.execute(&plan);
        assert_eq!(10000, *cpu.regs.pc);
    }

    fn sample_cpu(program: &[u8]) -> CPU<z80::MemoryBank> {
        // Test code runs in debug mode, which is highly inefficient.
        // Use a low CPU frequency to avoid panics due to slow emulation.
        let mut input = program;
        CPU::new(z80::MemoryBank::from_data(&mut input).unwrap())
    }
}
