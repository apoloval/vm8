extern crate vm8;

use vm8::clock::{Clock, Frequency, FrequencyStats};
use vm8::cpu;
use vm8::bus::{self, Bus};
use vm8::cpu::{Processor};
use vm8::cpu::z80;
use vm8::mem;

const MAX_CYCLES: usize = 10_000_000;
const ITERATIONS: usize = 500;

fn main() {
    let program = &[
        0x0c,               // INC C
        0x0d,               // DEC C
        0xc3, 0x00, 0x00,   // JP 0000h
    ];
    let mut input: &[u8] = program;
    let mem = mem::MemoryBank::from_data(&mut input).unwrap().share();
    let io = Bus::<u8, u8>::share(bus::Dead);
    let mut cpu = z80::CPU::new(z80::Options::default(), mem, io);

    let plan = cpu::ExecutionPlan::with_max_cycles(MAX_CYCLES);
    let mut clock = Clock::new(Frequency::from_mhz(3.54));
    let mut reports = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let exec_res = cpu.execute(&plan);
        let report = clock.sync(exec_res.total_cycles, false);
        reports.push(report.native_freq);
    }

    let stats = FrequencyStats::evaluate(reports);

    println!("Program executed {} iterations of {} cycles)", ITERATIONS, MAX_CYCLES);
    println!("   AVG frequency: {}", stats.avg);
    println!("   P95 frequency: {}", stats.p95);
    println!("   P99 frequency: {}", stats.p99);
    println!("   Max frequency: {}", stats.max);
    println!("   Min frequency: {}", stats.min);
}
