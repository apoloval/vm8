extern crate hemu;

use std::io;
use std::io::{Read, Write};

use hemu::bus::{Address, Memory};
use hemu::cpu;
use hemu::cpu::z80;

struct ComputerMem {
    data: [u8; 64*1024],
}

impl ComputerMem {
    fn new(program: &[u8]) -> ComputerMem {
        let mut mem = ComputerMem { data: [0; 64*1024] };
        {
            let mut input = program;
            let mut output: &mut[u8] = &mut mem.data;
            io::copy(&mut input, &mut output).unwrap();
        }
        mem
    }
}

impl Memory for ComputerMem {
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

fn main() {
    let program = [0x00];
    let mem = ComputerMem::new(&program);
    let mut cpu = z80::CPU::new(mem, cpu::Frequency::from_mhz(48.0), 10000);
    for _ in 0..10000 {
        cpu.exec_step();
    }
    let f = cpu.clock().native_freq().unwrap();
    println!("Program executed successfully (current native freq is {})", f);
}
