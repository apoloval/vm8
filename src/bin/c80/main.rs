extern crate hemu;

use hemu::clock::{Clock, Frequency};
use hemu::cpu;
use hemu::cpu::{Processor};
use hemu::cpu::z80;

const MAX_CYCLES: usize = 10_000_000;

fn main() {
    let program = &[
        0x0c,               // INC C
        0x0d,               // DEC C
        0xc3, 0x00, 0x00,   // JP 0000h
    ];
    let mut input: &[u8] = program;
    let mem = z80::MemoryBank::from_data(&mut input).unwrap();
    let mut cpu = z80::CPU::new(mem);

    let plan = cpu::ExecutionPlan::with_max_cycles(MAX_CYCLES);
    let mut clock = Clock::new(Frequency::from_mhz(3.54));
    let exec_res = cpu.execute(&plan);
    let native_freq = clock.sync(exec_res.total_cycles);

    println!("Program executed {} cycles at native freq of {})", exec_res.total_cycles, native_freq);
}
