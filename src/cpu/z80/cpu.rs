use std::time::Instant;
use std::thread;

use bus::Memory16;

use bus;
use cpu::Frequency;
use cpu::z80::inst::{Context, Inst};
use cpu::z80::regs::Registers;

pub struct CPU<M: Memory16> {
    mem: M,
    regs: Registers,
    freq: Frequency,
}

impl<M: Memory16> Context for CPU<M> {
    type Mem = M;
    fn regs(&self) -> &Registers { &self.regs }
    fn regs_mut(&mut self) -> &mut Registers { &mut self.regs }
    fn mem(&self) -> &M { &self.mem }
    fn mem_mut(&mut self) -> &mut M { &mut self.mem }
}

impl<M: Memory16> CPU<M> {
    pub fn new(mem: M, freq: Frequency) -> CPU<M> {
        CPU {
            mem: mem,
            regs: Registers::new(),
            freq: freq,
        }
    }
    pub fn exec_step(&mut self) {
        let t0 = Instant::now();
        let inst = self.decode_inst();
        let cycles = inst.exec(self);
        let t1 = t0 + (self.freq.period() * cycles as u32);
        let wait = t1 - Instant::now();
        thread::sleep(wait);
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
    use std::io::{Read, Write};

    use bus::{Addr16, Memory};
    use cpu::z80::inst::Inst;

    use super::*;

    #[test]
    fn exec_nop() {
        let mut cpu = sample_cpu();
        cpu.exec_inst(&Inst::Nop);
        assert_eq!(Addr16::from(0x0001), cpu.regs.pc());
    }

    struct SampleMem {
        data: [u8; 64*1024],
    }

    impl SampleMem {
        fn new() -> SampleMem {
            SampleMem { data: [0; 64*1024] }
        }
    }

    impl Memory for SampleMem {
        type Addr = Addr16;

        fn read(&self, addr: Addr16, buf: &mut[u8]) {
            let from = u16::from(addr) as usize;
            let mut input: &[u8] = &self.data[from..];
            input.read(buf).unwrap();
        }

        fn write(&mut self, addr: Addr16, buf: &[u8]) {
            let from = u16::from(addr) as usize;
            let mut input: &mut [u8] = &mut self.data[from..];
            input.write(buf).unwrap();
        }
    }

    fn sample_cpu() -> CPU<SampleMem> {
        CPU::new(SampleMem::new(), Frequency::from_mhz(20.0))
    }
}
