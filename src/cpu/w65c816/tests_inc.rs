use self::status::FlagExpectation;

use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case::native_8bit(
    "P.E:0,P.M:1,PC:A000,A:1112",                   // cpu
    "00A000:1A",                                    // bus
    0x1113,                                         // expected
    "Z:0,N:0",                                      // expected_flags_set
)]
#[case::native_8bit_negative(
    "P.E:0,P.M:1,PC:A000,A:117F",                   // cpu
    "00A000:1A",                                    // bus
    0x1180,                                         // expected
    "Z:0,N:1",                                      // expected_flags_set
)]
#[case::native_8bit_zero(
    "P.E:0,P.M:1,PC:A000,A:11FF",                   // cpu
    "00A000:1A",                                    // bus
    0x1100,                                         // expected
    "Z:1,N:0",                                      // expected_flags_set
)]
#[case::native_16bit(
    "P.E:0,P.M:0,PC:A000,A:11FF",                   // cpu
    "00A000:1A",                                    // bus
    0x1200,                                         // expected
    "Z:0,N:0",                                      // expected_flags_set
)]
#[case::native_16bit_negative(
    "P.E:0,P.M:0,PC:A000,A:7FFF",                   // cpu
    "00A000:1A",                                    // bus
    0x8000,                                         // expected
    "Z:0,N:1",                                      // expected_flags_set
)]
#[case::native_16bit_zero(
    "P.E:0,P.M:0,PC:A000,A:FFFF",                   // cpu
    "00A000:1A",                                    // bus
    0x0000,                                         // expected
    "Z:1,N:0",                                      // expected_flags_set
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
#[case::accumulator(
    "PC:A000",                                      // cpu
    "00A000:1A",                                    // bus
    ("INC", ""),                                    // expected
    0xA001,                                         // expected_pc
)]
#[case::direct(
    "PC:A000",                                      // cpu
    "00A000:E6AB",                                  // bus
    ("INC", "$AB"),                                 // expected
    0xA002,                                         // expected_pc
)]
#[case::absolute(
    "PC:A000",                                      // cpu
    "00A000:EECDAB",                                // bus
    ("INC", "$ABCD"),                               // expected
    0xA003,                                         // expected_pc
)]
#[case::direct_indexed_x(
    "PC:A000",                                      // cpu
    "00A000:F6AB",                                  // bus
    ("INC", "$AB,X"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::absolute_indexed_x(
    "PC:A000",                                      // cpu
    "00A000:FECDAB",                                // bus
    ("INC", "$ABCD,X"),                             // expected
    0xA003,                                         // expected_pc
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
