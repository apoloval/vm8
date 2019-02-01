use std::boxed::Box;

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

pub struct CPU {
    opts: Options,
    mem: Box<dyn bus::Memory>,
    io: Box<dyn bus::IO>,
    regs: Registers,
    alu: ALU,
}

impl Context for CPU {
    type Mem = Box<dyn bus::Memory>;
    type IO = Box<dyn bus::IO>;

    fn alu(&self) -> &ALU { &self.alu }
    fn regs(&self) -> &Registers { &self.regs }
    fn regs_mut(&mut self) -> &mut Registers { &mut self.regs }
    fn mem(&self) -> &Box<dyn bus::Memory> { &self.mem }
    fn mem_mut(&mut self) -> &mut Box<dyn bus::Memory> { &mut self.mem }
    fn io(&self) -> &Box<dyn bus::IO> { &self.io }
    fn io_mut(&mut self) -> &mut Box<dyn bus::IO> { &mut self.io }
}

impl Processor for CPU {
    fn execute(&mut self, plan: &ExecutionPlan) -> ExecutionResult {
        let mut result = ExecutionResult::default();
        while !plan.is_completed(&result) {
            result.total_cycles += exec_step(self) + self.opts.m1_wait_cycles;
            result.total_instructions += 1;
        }
        result
    }
}

impl CPU {
    pub fn new(opts: Options, mem: Box<dyn bus::Memory>, io: Box<dyn bus::IO>) -> Self {
        Self {
            opts: opts,
            mem: mem,
            io: io,
            regs: Registers::new(),
            alu: ALU::new(),
        }
    }
}
