use crate::cpu::{ExecutionPlan, ExecutionResult, Processor};
use crate::cpu::z80::alu::ALU;
use crate::cpu::z80::exec::{Context, exec_step};
use crate::cpu::z80::mem::MemoryBus;
use crate::cpu::z80::reg::Registers;

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
    alu: ALU,
}

impl<M: MemoryBus> Context for CPU<M> {
    type Mem = M;
    fn alu(&self) -> &ALU { &self.alu }
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
            alu: ALU::new(),
        }
    }
}
