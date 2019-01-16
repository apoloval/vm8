#[macro_use] mod macros;

mod alu;
mod device;
mod exec;
mod mem;
mod reg;

pub use self::device::{CPU, Options};
pub use self::mem::MemoryBank;

#[cfg(all(feature = "nightly", test))]
mod bench {
    use std::io::Write;
    use test;
    use test::Bencher;

    use crate::cpu;
    use crate::cpu::Processor;
    use crate::cpu::z80;

    #[bench]
    fn bench_exec_1000_cycles_of_sample_program(b: &mut Bencher) {
        bench_sample_program(b, 1_000);
    }

    #[bench]
    fn bench_exec_10_million_cycles_of_sample_program(b: &mut Bencher) {
        bench_sample_program(b, 10_000_000);
    }

    fn bench_sample_program(b: &mut Bencher, cycles: usize) {
        let mut program = Vec::new();
        program.write(&inst!(INC C)).unwrap();
        program.write(&inst!(DEC C)).unwrap();
        program.write(&inst!(JP 0x0000)).unwrap();

        let mem = z80::MemoryBank::from_data(&mut &program[..]).unwrap();
        let mut cpu = z80::CPU::new(z80::Options::default(), mem);
        let plan = cpu::ExecutionPlan::with_max_cycles(cycles);

        b.iter(|| {
            test::black_box(cpu.execute(&plan));
        })
    }
}
