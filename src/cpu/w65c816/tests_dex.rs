use self::status::FlagExpectation;

use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case::native_8bit(
    "P.E:0,P.X:1,PC:A000,X:12",                     // cpu
    "00A000:CA",                                    // bus
    0x0011,                                         // expected
    "Z:0,N:0",                                      // expected_flags_set
)]
#[case::native_8bit_negative(
    "P.E:0,P.X:1,PC:A000,X:00",                     // cpu
    "00A000:CA",                                    // bus
    0x00FF,                                         // expected
    "Z:0,N:1",                                      // expected_flags_set
)]
#[case::native_8bit_zero(
    "P.E:0,P.X:1,PC:A000,X:01",                     // cpu
    "00A000:CA",                                    // bus
    0x0000,                                         // expected
    "Z:1,N:0",                                      // expected_flags_set
)]
#[case::native_16bit(
    "P.E:0,P.X:0,PC:A000,X:1100",                   // cpu
    "00A000:CA",                                    // bus
    0x10FF,                                         // expected
    "Z:0,N:0",                                      // expected_flags_set
)]
#[case::native_16bit_negative(
    "P.E:0,P.X:0,PC:A000,X:0000",                   // cpu
    "00A000:CA",                                    // bus
    0xFFFF,                                         // expected
    "Z:0,N:1",                                      // expected_flags_set
)]
#[case::native_16bit_zero(
    "P.E:0,P.X:0,PC:A000,X:0001",                   // cpu
    "00A000:CA",                                    // bus
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

    assert::index_x(&cpu, expected);
    expected_flags.assert(cpu.regs.p());
}

#[rstest]
#[case::implicit(
    "PC:A000",                                      // cpu
    "00A000:CA",                                    // bus
    ("DEX", ""),                                    // expected
    0xA001,                                         // expected_pc
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