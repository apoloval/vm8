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
    ($cpu:expr, A <- $eval:expr) => { $cpu.regs_mut().set_a(cpu_eval!($cpu, $eval)) };
    ($cpu:expr, F <- $eval:expr) => { $cpu.regs_mut().set_flags(cpu_eval!($cpu, $eval)) };
    ($cpu:expr, B <- $eval:expr) => { $cpu.regs_mut().set_b(cpu_eval!($cpu, $eval)) };
    ($cpu:expr, C <- $eval:expr) => { $cpu.regs_mut().set_c(cpu_eval!($cpu, $eval)) };
    ($cpu:expr, D <- $eval:expr) => { $cpu.regs_mut().set_d(cpu_eval!($cpu, $eval)) };
    ($cpu:expr, E <- $eval:expr) => { $cpu.regs_mut().set_e(cpu_eval!($cpu, $eval)) };
    ($cpu:expr, H <- $eval:expr) => { $cpu.regs_mut().set_h(cpu_eval!($cpu, $eval)) };
    ($cpu:expr, L <- $eval:expr) => { $cpu.regs_mut().set_l(cpu_eval!($cpu, $eval)) };
    ($cpu:expr, AF <-> AF_) => { $cpu.regs_mut().swap_af() };
    ($cpu:expr, AF <- $eval:expr) => { $cpu.regs_mut().set_af(cpu_eval!($cpu, $eval)) };
    ($cpu:expr, AF_ <- $eval:expr) => { $cpu.regs_mut().set_af_(cpu_eval!($cpu, $eval)) };
    ($cpu:expr, BC <- $eval:expr) => { $cpu.regs_mut().set_bc(cpu_eval!($cpu, $eval)) };
    ($cpu:expr, DE <- $eval:expr) => { $cpu.regs_mut().set_de(cpu_eval!($cpu, $eval)) };
    ($cpu:expr, HL <- $eval:expr) => { $cpu.regs_mut().set_hl(cpu_eval!($cpu, $eval)) };
    ($cpu:expr, SP <- $eval:expr) => { $cpu.regs_mut().set_sp(cpu_eval!($cpu, $eval)) };
    ($cpu:expr, PC +<- $eval:expr) => { $cpu.regs_mut().inc_pc8(cpu_eval!($cpu, $eval)) };
    ($cpu:expr, PC ++<- $eval:expr) => { $cpu.regs_mut().inc_pc(cpu_eval!($cpu, $eval)) };
    ($cpu:expr, PC <- $eval:expr) => { $cpu.regs_mut().set_pc(cpu_eval!($cpu, $eval)) };
    ($cpu:expr, PC++) => { $cpu.regs_mut().inc_pc(1) };

    ($cpu:expr, ($reg:ident) <- $eval:expr) => { 
        let addr = cpu_eval!($cpu, $reg);
        let val = cpu_eval!($cpu, $eval);
        $cpu.mem_mut().write_to(addr, val)
    };

    ($cpu:expr, ($reg:ident) as u16 <- $eval:expr) => { 
        let addr = cpu_eval!($cpu, $reg);
        let val = cpu_eval!($cpu, $eval);
        $cpu.mem_mut().write_word_to::<LittleEndian>(addr, val) 
    };

    ($cpu:expr, ($addr:expr) <- $eval:expr) => { $cpu.mem_mut().write_to($addr, cpu_eval!($cpu, $eval)) };
    ($cpu:expr, ($addr:expr) as u16 <- $eval:expr) => { $cpu.mem_mut().write_word_to::<LittleEndian>($addr, cpu_eval!($cpu, $eval)) };

    ($cpu:expr, A) => { $cpu.regs().a() };
    ($cpu:expr, F) => { $cpu.regs().flags() };
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
    ($cpu:expr, PC) => { $cpu.regs().pc() };
    ($cpu:expr, ($reg:ident) as u16) => { $cpu.mem().read_word_from::<LittleEndian>(cpu_eval!($cpu, $reg)) };
    ($cpu:expr, ($reg:ident)) => { $cpu.mem().read_from(cpu_eval!($cpu, $reg)) };

    ($cpu:expr, ($addr:expr) as u16) => { $cpu.mem().read_word_from::<LittleEndian>($addr) };

    ($cpu:expr, ($addr:expr)) => { $cpu.mem().read_from($addr) };

    ($cpu:expr, $val:expr) => { $val };
}