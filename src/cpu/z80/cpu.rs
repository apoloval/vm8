use cpu::{ExecutionPlan, ExecutionResult, Processor};
use cpu::z80::{Context, MemoryBus, Registers, exec_step};

pub struct Options {
    pub m1_wait_cycles: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            m1_wait_cycles: 0,
        }
    }
}

pub struct CPU<M: MemoryBus> {
    opts: Options,
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
            result.total_cycles += exec_step(self) + self.opts.m1_wait_cycles;
            result.total_instructions += 1;
        }
        result
    }
}

impl<M: MemoryBus> CPU<M> {
    pub fn new(opts: Options, mem: M) -> Self {
        Self {
            opts: opts,
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
        CPU::new(Options::default(), z80::MemoryBank::from_data(&mut input).unwrap())
    }
}
