extern crate vm8;

use std::boxed::Box;

use vm8::clock::{Clock, Frequency};
use vm8::cpu;
use vm8::bus;
use vm8::cpu::{Processor};
use vm8::cpu::z80;
use vm8::mem;

const MAX_CYCLES: usize = 10_000_000;

fn main() {
    let program = &[
        0x0c,               // INC C
        0x0d,               // DEC C
        0xc3, 0x00, 0x00,   // JP 0000h
    ];
    let mut input: &[u8] = program;
    let mem = Box::new(mem::MemoryBank::from_data(&mut input).unwrap());
    let io = Box::new(bus::Dead::new());
    let mut cpu = z80::CPU::new(z80::Options::default(), mem, io);

    let plan = cpu::ExecutionPlan::with_max_cycles(MAX_CYCLES);
    let mut clock = Clock::new(Frequency::from_mhz(3.54));
    for _ in 0..10 {
        let exec_res = cpu.execute(&plan);
        let report = clock.sync(exec_res.total_cycles, false);
        println!(
            "Program executed {} cycles in {:?} at native freq of {})",
            exec_res.total_cycles, report.real_duration, report.native_freq);
    }
}
