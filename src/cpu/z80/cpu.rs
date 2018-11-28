use cpu::{ExecutionPlan, ExecutionResult, Processor};
use cpu::z80::{Context, MemoryBus, Registers, exec_step};
use cpu::z80::alu::ALU;

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

// Evaluate the given expression in the context of the CPU
macro_rules! cpu_eval {
    ($cpu:expr, A) => { $cpu.regs().a() };
    ($cpu:expr, B) => { $cpu.regs().b() };
    ($cpu:expr, C) => { $cpu.regs().c() };
    ($cpu:expr, D) => { $cpu.regs().d() };
    ($cpu:expr, E) => { $cpu.regs().e() };
    ($cpu:expr, H) => { $cpu.regs().h() };
    ($cpu:expr, L) => { $cpu.regs().l() };
    ($cpu:expr, AF) => { $cpu.regs().af() };
    ($cpu:expr, AF_) => { $cpu.regs().af_() };
    ($cpu:expr, BC) => { $cpu.regs().bc() };
    ($cpu:expr, DE) => { $cpu.regs().de() };
    ($cpu:expr, HL) => { $cpu.regs().hl() };
    ($cpu:expr, SP) => { $cpu.regs().sp() };
    ($cpu:expr, ($reg:tt) as u16) => { $cpu.mem().read_word_from::<LittleEndian>(cpu_eval!($cpu, $reg)) };
    ($cpu:expr, ($reg:tt)) => { $cpu.mem().read_from(cpu_eval!($cpu, $reg)) };
    ($cpu:expr, $val:tt) => { $val };
}