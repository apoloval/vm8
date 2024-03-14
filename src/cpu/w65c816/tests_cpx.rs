use self::status::FlagExpectation;

use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case::native_8bit_less_than(
    "P.E:0,P.M:1,PC:A000,X:1123",                   // cpu
    "00A000:E045",                                  // bus
    "C:0,Z:0,N:1",                                  // expected_flags_set
)]
#[case::native_8bit_greater_than(
    "P.E:0,P.M:1,PC:A000,X:1145",                   // cpu
    "00A000:E023",                                  // bus
    "C:1,Z:0,N:0",                                  // expected_flags_set
)]
#[case::native_8bit_equal(
    "P.E:0,P.M:1,PC:A000,X:1142",                   // cpu
    "00A000:E042",                                  // bus
    "C:1,Z:1,N:0",                                  // expected_flags_set
)]
#[case::native_8bit_neg(
    "P.E:0,P.M:1,PC:A000,X:1182",                   // cpu
    "00A000:E001",                                  // bus
    "C:1,Z:0,V:0,N:1",                              // expected_flags_set
)]
#[case::native_16bit_less_than(
    "P.E:0,P.M:0,PC:A000,X:1234",                   // cpu
    "00A000:E07856",                                // bus
    "C:0,Z:0,V:0,N:1",                              // expected_flags_set
)]
#[case::native_16bit_greater_than(
    "P.E:0,P.M:0,PC:A000,X:5678",                   // cpu
    "00A000:E03412",                                // bus
    "C:1,Z:0,V:0,N:0",                              // expected_flags_set
)]
#[case::native_16bit_equal(
    "P.E:0,P.M:0,PC:A000,X:1234",                   // cpu
    "00A000:E03412",                                // bus
    "C:1,Z:1,V:0,N:0",                              // expected_flags_set
)]
#[case::native_16bit_neg(
    "P.E:0,P.M:0,PC:A000,X:ABCD",                   // cpu
    "00A000:E03412",                                // bus
    "C:1,Z:0,V:0,N:1",                              // expected_flags_set
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
    "00A000:EC5634",                                // bus
    ("CPX", "$3456"),                               // expected
    0xA003,                                         // expected_pc
)]
#[case::direct(
    "PC:A000",                                      // cpu
    "00A000:E404",                                  // bus
    ("CPX", "$04"),                                 // expected
    0xA002,                                         // expected_pc
)]
#[case::immediate(
    "PC:A000",                                      // cpu
    "00A000:E0FFFF",                                // bus
    ("CPX", "#$FFFF"),                              // expected
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
