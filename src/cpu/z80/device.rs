use crate::cpu::{ExecutionPlan, ExecutionResult, Processor};
use crate::cpu::z80::alu::ALU;
use crate::cpu::z80::bus;
use crate::cpu::z80::exec::{Context, exec_step};
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

pub struct CPU<Mem: bus::Memory, IO: bus::IO> {
    opts: Options,
    mem: Mem,
    io: IO,
    regs: Registers,
    alu: ALU,
}

impl<Mem: bus::Memory, IO: bus::IO> Context for CPU<Mem, IO> {
    type Mem = Mem;
    type IO = IO;

    fn alu(&self) -> &ALU { &self.alu }
    fn regs(&self) -> &Registers { &self.regs }
    fn regs_mut(&mut self) -> &mut Registers { &mut self.regs }
    fn mem(&mut self) -> &mut Mem { &mut self.mem }
    fn io(&mut self) -> &mut IO { &mut self.io }
}

impl<Mem: bus::Memory, IO: bus::IO> Processor for CPU<Mem, IO> {
    fn execute(&mut self, plan: &ExecutionPlan) -> ExecutionResult {
        let mut result = ExecutionResult::default();
        while !plan.is_completed(&result) {
            result.total_cycles += exec_step(self) + self.opts.m1_wait_cycles;
            result.total_instructions += 1;
        }
        result
    }
}

impl<Mem: bus::Memory, IO: bus::IO> CPU<Mem, IO> {
    pub fn new(opts: Options, mem: Mem, io: IO) -> Self {
        Self {
            opts: opts,
            mem: mem,
            io: io,
            regs: Registers::new(),
            alu: ALU::new(),
        }
    }
}
