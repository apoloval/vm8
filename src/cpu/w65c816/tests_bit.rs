use self::status::FlagExpectation;

use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case::emulation(
    "P.E:1,PC:A000,A:1122",                         // cpu
    "00A000:89F0",                                  // bus
    "Z:0,N:0,V:0",                                  // expected_flags_set
)]
#[case::native_8bit(
    "P.E:0,P.M:1,PC:A000,A:1122",                   // cpu
    "00A000:89F0",                                  // bus
    "Z:0,N:0,V:0",                                  // expected_flags_set
)]
#[case::native_16bit(
    "P.E:0,P.M:0,PC:A000,A:1122",                   // cpu
    "00A000:89F00F",                                // bus
    "Z:0,N:0,V:0",                                  // expected_flags_set
)]
#[case::native_8bit_zero(
    "P.E:0,P.M:1,PC:A000,A:1122",                   // cpu
    "00A000:8900",                                  // bus
    "Z:1,N:0,V:0",                                  // expected_flags_set
)]
#[case::native_16bit_zero(
    "P.E:0,P.M:0,PC:A000,A:1122",                   // cpu
    "00A000:890000",                                // bus
    "Z:1,N:0,V:0",                                  // expected_flags_set
)]
#[case::native_8bit_neg(
    "P.E:0,P.M:1,DP:4000,PC:A000,A:110F",           // cpu
    "004012:8F,00A000:2412",                        // bus
    "Z:0,N:1,V:0",                                  // expected_flags_set
)]
#[case::native_8bit_neg_immediate(
    "P.E:0,P.M:1,PC:A000,A:110F",                   // cpu
    "00A000:898F",                                  // bus
    "Z:0,N:0,V:0",                                  // expected_flags_set
)]
#[case::native_16bit_neg(
    "P.E:0,P.M:0,DP:4000,PC:A000,A:00FF",           // cpu
    "004012:FF8F,00A000:2412",                      // bus
    "Z:0,N:1,V:0",                                  // expected_flags_set
)]
#[case::native_16bit_neg_immediate(
    "P.E:0,P.M:0,PC:A000,A:00FF",                   // cpu
    "00A000:89FF8F",                                // bus
    "Z:0,N:0,V:0",                                  // expected_flags_set
)]
#[case::native_8bit_neg_second(
    "P.E:0,P.M:1,DP:4000,PC:A000,A:110F",           // cpu
    "004012:4F,00A000:2412",                        // bus
    "Z:0,N:0,V:1",                                  // expected_flags_set
)]
#[case::native_8bit_neg_second_immediate(
    "P.E:0,P.M:1,PC:A000,A:110F",                   // cpu
    "00A000:894F",                                  // bus
    "Z:0,N:0,V:0",                                  // expected_flags_set
)]
#[case::native_16bit_neg_second(
    "P.E:0,P.M:0,DP:4000,PC:A000,A:00FF",           // cpu
    "004012:FF4F,00A000:2412",                      // bus
    "Z:0,N:0,V:1",                                  // expected_flags_set
)]
#[case::native_16bit_neg_second_immediate(
    "P.E:0,P.M:0,PC:A000,A:00FF",                   // cpu
    "00A000:89FF4F",                                // bus
    "Z:0,N:0,V:0",                                  // expected_flags_set
)]
fn test_results(
    #[case] mut cpu: CPU,
    #[case] mut bus: bus::Fake,
    #[case] expected_flags: FlagExpectation,
) {
    cpu.step(&mut bus, &mut NullReporter);

    expected_flags.assert(cpu.regs.p());
}

#[rstest]
#[case::absolute(
    "PC:A000",                                      // cpu
    "00A000:2C5634",                                // bus
    ("BIT", "$3456"),                               // expected
    0xA003,                                         // expected_pc
)]
#[case::absolute_indexed_x(
    "PC:A000",                                      // cpu
    "00A000:3C5634",                                // bus
    ("BIT", "$3456,X"),                             // expected
    0xA003,                                         // expected_pc
)]
#[case::direct(
    "PC:A000",                                      // cpu
    "00A000:2404",                                  // bus
    ("BIT", "$04"),                                 // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indexed_x(
    "PC:A000",                                      // cpu
    "00A000:3404",                                  // bus
    ("BIT", "$04,X"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::immediate(
    "PC:A000",                                      // cpu
    "00A000:89FFFF",                                // bus
    ("BIT", "#$FFFF"),                              // expected
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
