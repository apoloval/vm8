use bus::Memory;

use bus;
use cpu::{Clock, Frequency};
use cpu::z80::inst::{Context, Inst};
use cpu::z80::regs::Registers;

pub struct CPU<M: Memory> {
    mem: M,
    regs: Registers,
    clock: Clock,
}

impl<M: Memory> Context for CPU<M> {
    type Mem = M;
    fn regs(&self) -> &Registers { &self.regs }
    fn regs_mut(&mut self) -> &mut Registers { &mut self.regs }
    fn mem(&self) -> &M { &self.mem }
    fn mem_mut(&mut self) -> &mut M { &mut self.mem }
}

impl<M: Memory> CPU<M> {
    pub fn new(mem: M, freq: Frequency) -> CPU<M> {
        CPU {
            mem: mem,
            regs: Registers::new(),
            clock: Clock::new(freq),
        }
    }

    pub fn clock(&self) -> &Clock { &self.clock }

    pub fn exec_step(&mut self) {
        let inst = self.decode_inst();
        let cycles = inst.exec(self);
        self.clock.walk(cycles);
    }

    pub fn exec_inst(&mut self, inst: &Inst) {
        inst.exec(self);
    }

    fn decode_inst(&mut self) -> Inst {
        let mut mread = bus::read_from(&self.mem, self.regs.pc());
        Inst::decode(&mut mread).expect("memory read should never fail")
    }
}

#[cfg(test)]
mod test {
    use std::io;
    use std::io::{Read, Write};

    use bus::{Address, Memory};

    use super::*;

    #[test]
    fn exec_nop() {
        let mut cpu = sample_cpu(&[0x00]);
        for _ in 0..10000 {
            cpu.exec_step();
        }
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
        fn read(&self, addr: Address, buf: &mut[u8]) {
            let from = u16::from(addr) as usize;
            let mut input: &[u8] = &self.data[from..];
            input.read(buf).unwrap();
        }

        fn write(&mut self, addr: Address, buf: &[u8]) {
            let from = u16::from(addr) as usize;
            let mut input: &mut [u8] = &mut self.data[from..];
            input.write(buf).unwrap();
        }
    }

    fn sample_cpu(program: &[u8]) -> CPU<SampleMem> {
        // Test code runs in debug mode, which is highly inefficient.
        // Use a low CPU frequency to avoid panics due to slow emulation.
        CPU::new(SampleMem::new(program), Frequency::from_khz(100.0))
    }
}
