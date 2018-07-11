extern crate hemu;

use hemu::bus::{Address, MemoryBank, MemoryController};
use hemu::cpu;
use hemu::cpu::z80;

struct ComputerMem {
    rom: MemoryBank,
    ram: MemoryBank,
}

impl ComputerMem {
    fn new(program: &[u8]) -> ComputerMem {
        let mut rom = MemoryBank::with_size(16 * 1024);
        let ram = MemoryBank::with_size(64 * 1024);
        rom.set_data(program).expect("program bytes are written");
        rom.set_readonly(true);
        ComputerMem { rom, ram }
    }
}

impl MemoryController for ComputerMem {
    fn bank(&self, addr: Address) -> Option<&MemoryBank> {
        match usize::from(addr) {
            0x0000 ... 0x3fff => Some(&self.rom),
            0x4000 ... 0xffff => Some(&self.ram),
            _ => None,
        }
    }

    fn bank_mut(&mut self, addr: Address) -> Option<&mut MemoryBank> {
        match usize::from(addr) {
            0x4000 ... 0xffff => Some(&mut self.ram),
            _ => None,
        }
    }
}

fn main() {
    let program = &[
        0x0c,               // INC C
        0x0d,               // DEC C
        0xc3, 0x00, 0x00,   // JP 0000h
    ];
    let mem = ComputerMem::new(program);
    let mut cpu = z80::CPU::new(mem, cpu::Frequency::from_mhz(60.0));
    for _ in 0..1_000_000 {
        cpu.exec_step();
    }
    let f = cpu.clock().native_freq().unwrap();
    println!("Program executed successfully (current native freq is {})", f);
}
