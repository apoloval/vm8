extern crate hemu;

use hemu::bus::MemoryBank;
use hemu::cpu;
use hemu::cpu::z80;

fn main() {
    let program = &[
        0x0c,               // INC C
        0x0d,               // DEC C
        0xc3, 0x00, 0x00,   // JP 0000h
    ];
    let mut mem = MemoryBank::with_size(64*1024);
    mem.set_data(program).expect("program bytes are written");
    let mut cpu = z80::CPU::new(mem, cpu::Frequency::from_mhz(60.0));
    for _ in 0..1_000_000 {
        cpu.exec_step();
    }
    let f = cpu.clock().native_freq().unwrap();
    println!("Program executed successfully (current native freq is {})", f);
}
