extern crate hemu;

use std::time::Instant;

use hemu::bus::MemoryBank;
use hemu::cpu;
use hemu::cpu::{Processor};
use hemu::cpu::z80;

const MAX_CYCLES: usize = 2_000_000;

fn main() {
    let program = &[
        0x0c,               // INC C
        0x0d,               // DEC C
        0xc3, 0x00, 0x00,   // JP 0000h
    ];
    let mut mem = MemoryBank::with_size(64*1024);
    mem.set_data(program).expect("program bytes are written");
    let mut cpu = z80::CPU::new(mem);

    let plan = cpu::ExecutionPlan::with_max_cycles(MAX_CYCLES);
    let t0 = Instant::now();
    let exec_res = cpu.execute(&plan);
    let t1 = Instant::now();
    let duration = t1 - t0;
    let duration_secs: f64 = duration.as_secs() as f64 + (duration.subsec_nanos() as f64 / 1_000_000_000.0);
    let freq = exec_res.total_cycles as f64 / duration_secs;

    println!("Program executed {} cycles in {}s ({}Mhz)", exec_res.total_cycles, duration_secs, freq / 1_000_000.0);
}
