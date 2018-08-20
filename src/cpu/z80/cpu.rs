use bus::Memory;
use cpu::{ExecutionPlan, ExecutionResult, Processor};
use cpu::z80::inst::{Context, Inst, decode};
use cpu::z80::reg::Registers;

pub struct CPU<M: Memory> {
    mem: M,
    regs: Registers,
}

impl<M: Memory> Context for CPU<M> {
    type Mem = M;
    fn regs(&self) -> &Registers { &self.regs }
    fn regs_mut(&mut self) -> &mut Registers { &mut self.regs }
    fn mem(&self) -> &M { &self.mem }
    fn mem_mut(&mut self) -> &mut M { &mut self.mem }
}

impl<M: Memory> Processor for CPU<M> {
    fn execute(&mut self, plan: &ExecutionPlan) -> ExecutionResult {
        let mut result = ExecutionResult::default();
        while !plan.is_completed(&result) {
            let inst = self.decode_inst();
            result.total_cycles += inst.exec(self);
            result.total_instructions += 1;
        }
        result
    }
}

impl<M: Memory> CPU<M> {
    pub fn new(mem: M) -> Self {
        Self {
            mem: mem,
            regs: Registers::new(),
        }
    }

    fn decode_inst(&mut self) -> Inst {
        decode(&self.mem, self.regs.pc())
    }
}

#[cfg(test)]
mod test {
    use std::io;

    use bus::{Address, Memory};

    use super::*;

    #[test]
    fn exec_nop() {
        let mut cpu = sample_cpu(&[0x00]);
        let plan = ExecutionPlan::with_max_instructions(10000);
        cpu.execute(&plan);
        assert_eq!(Address::from(10000), cpu.regs.pc());
    }

    struct SampleMem {
        data: [u8; 64*1024],
    }

    impl SampleMem {
        fn new(program: &[u8]) -> SampleMem {
            let mut mem = SampleMem { data: [0; 64*1024] };
            {
                let mut input = program;
                let mut output: &mut[u8] = &mut mem.data;
                io::copy(&mut input, &mut output).unwrap();
            }
            mem
        }
    }

    impl Memory for SampleMem {
        fn read_byte(&self, addr: Address) -> u8 {
            self.data[usize::from(addr)]
        }

        fn write_byte(&mut self, addr: Address, val: u8) {
            self.data[usize::from(addr)] = val;
        }
    }

    fn sample_cpu(program: &[u8]) -> CPU<SampleMem> {
        // Test code runs in debug mode, which is highly inefficient.
        // Use a low CPU frequency to avoid panics due to slow emulation.
        CPU::new(SampleMem::new(program))
    }
}
