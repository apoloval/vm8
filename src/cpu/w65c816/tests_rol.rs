use self::status::FlagExpectation;

use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case::emulation(
    "P.E:1,PC:A000,A:1135",                         // cpu
    "00A000:2A",                                    // bus
    0x116A,                                         // expected
    "Z:0,N:0,C:0",                                  // expected_flags_set
)]
#[case::native_8bit(
    "P.E:0,P.M:1,PC:A000,A:1135",                   // cpu
    "00A000:2A",                                    // bus
    0x116A,                                         // expected
    "Z:0,N:0,C:0",                                  // expected_flags_set
)]
#[case::native_8bit_carry_in(
    "P.E:0,P.M:1,P.C:1,PC:A000,A:1135",             // cpu
    "00A000:2A",                                    // bus
    0x116B,                                         // expected
    "Z:0,N:0,C:0",                                  // expected_flags_set
)]
#[case::native_8bit_carry_out(
    "P.E:0,P.M:1,PC:A000,A:1181",                   // cpu
    "00A000:2A",                                    // bus
    0x1102,                                         // expected
    "Z:0,N:0,C:1",                                  // expected_flags_set
)]
#[case::native_8bit_zero(
    "P.E:0,P.M:1,PC:A000,A:1100",                   // cpu
    "00A000:2A",                                    // bus
    0x1100,                                         // expected
    "Z:1,N:0,C:0",                                  // expected_flags_set
)]
#[case::native_8bit_neg(
    "P.E:0,P.M:1,PC:A000,A:1140",                   // cpu
    "00A000:2A",                                    // bus
    0x1180,                                         // expected
    "Z:0,N:1,C:0",                                  // expected_flags_set
)]
#[case::native_16bit(
    "P.E:0,P.M:0,PC:A000,A:2A35",                   // cpu
    "00A000:2A",                                    // bus
    0x546A,                                         // expected
    "Z:0,N:0,C:0",                                  // expected_flags_set
)]
#[case::native_16bit_carry_in(
    "P.E:0,P.M:0,P.C:1,PC:A000,A:2A35",             // cpu
    "00A000:2A",                                    // bus
    0x546B,                                         // expected
    "Z:0,N:0,C:0",                                  // expected_flags_set
)]
#[case::native_16bit_carry_out(
    "P.E:0,P.M:0,PC:A000,A:8001",                   // cpu
    "00A000:2A",                                    // bus
    0x0002,                                         // expected
    "Z:0,N:0,C:1",                                  // expected_flags_set
)]
#[case::native_16bit_neg(
    "P.E:0,P.M:0,PC:A000,A:4000",                   // cpu
    "00A000:2A",                                    // bus
    0x8000,                                         // expected
    "Z:0,N:1,C:0",                                  // expected_flags_set
)]
#[case::native_16bit_zero(
    "P.E:0,P.M:0,PC:A000,A:0000",                   // cpu
    "00A000:2A",                                    // bus
    0x0000,                                         // expected
    "Z:1,N:0,C:0",                                  // expected_flags_set
)]
fn test_results(
    #[case] mut cpu: CPU,
    #[case] mut bus: bus::Fake,
    #[case] expected: u16,
    #[case] expected_flags: FlagExpectation,
) {
    cpu.step(&mut bus, &mut NullReporter);

    assert::accum(&cpu, expected);
    expected_flags.assert(cpu.regs.p());
}

#[rstest]
#[case::absolute(
    "PC:A000",                                      // cpu
    "00A000:2E5634",                                // bus
    ("ROL", "$3456"),                               // expected
    0xA003,                                         // expected_pc
)]
#[case::absolute_indexed_x(
    "PC:A000",                                      // cpu
    "00A000:3E5634",                                // bus
    ("ROL", "$3456,X"),                             // expected
    0xA003,                                         // expected_pc
)]
#[case::accumulator(
    "PC:A000",                                      // cpu
    "00A000:2A",                                    // bus
    ("ROL", ""),                                    // expected
    0xA001,                                         // expected_pc
)]
#[case::direct(
    "PC:A000",                                      // cpu
    "00A000:2604",                                  // bus
    ("ROL", "$04"),                                 // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indexed_x(
    "PC:A000",                                      // cpu
    "00A000:3604",                                  // bus
    ("ROL", "$04,X"),                               // expected
    0xA002,                                         // expected_pc
)]
fn test_decoding(
    #[case] mut cpu: CPU,
    #[case] mut bus: bus::Fake,
    #[case] expected: (&'static str, &'static str),
    #[case] expected_pc: u16,
) {
    let mut reporter = ev::Retain::new();
    cpu.step(&mut bus, &mut reporter);

    let (expected_inst, expected_ops) = expected;
    reporter.assert_exec(expected_inst, expected_ops);
    assert::program_counter(&cpu, cpu.regs.pbr(), expected_pc);
}
