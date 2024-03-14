use self::status::FlagExpectation;

use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case::native_8bit_less_than(
    "P.E:0,P.M:1,PC:A000,A:1123",                   // cpu
    "00A000:C945",                                  // bus
    "C:0,Z:0,N:1",                                  // expected_flags_set
)]
#[case::native_8bit_greater_than(
    "P.E:0,P.M:1,PC:A000,A:1145",                   // cpu
    "00A000:C923",                                  // bus
    "C:1,Z:0,N:0",                                  // expected_flags_set
)]
#[case::native_8bit_equal(
    "P.E:0,P.M:1,PC:A000,A:1142",                   // cpu
    "00A000:C942",                                  // bus
    "C:1,Z:1,N:0",                                  // expected_flags_set
)]
#[case::native_8bit_neg(
    "P.E:0,P.M:1,PC:A000,A:1182",                   // cpu
    "00A000:C901",                                  // bus
    "C:1,Z:0,V:0,N:1",                              // expected_flags_set
)]
#[case::native_16bit_less_than(
    "P.E:0,P.M:0,PC:A000,A:1234",                   // cpu
    "00A000:C97856",                                // bus
    "C:0,Z:0,V:0,N:1",                              // expected_flags_set
)]
#[case::native_16bit_greater_than(
    "P.E:0,P.M:0,PC:A000,A:5678",                   // cpu
    "00A000:C93412",                                // bus
    "C:1,Z:0,V:0,N:0",                              // expected_flags_set
)]
#[case::native_16bit_equal(
    "P.E:0,P.M:0,PC:A000,A:1234",                   // cpu
    "00A000:C93412",                                // bus
    "C:1,Z:1,V:0,N:0",                              // expected_flags_set
)]
#[case::native_16bit_neg(
    "P.E:0,P.M:0,PC:A000,A:ABCD",                   // cpu
    "00A000:C93412",                                // bus
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
    "00A000:CD5634",                                // bus
    ("CMP", "$3456"),                               // expected
    0xA003,                                         // expected_pc
)]
#[case::absolute_indexed_x(
    "PC:A000",                                      // cpu
    "00A000:DD5634",                                // bus
    ("CMP", "$3456,X"),                             // expected
    0xA003,                                         // expected_pc
)]
#[case::absolute_indexed_y(
    "PC:A000",                                      // cpu
    "00A000:D95634",                                // bus
    ("CMP", "$3456,Y"),                             // expected
    0xA003,                                         // expected_pc
)]
#[case::absolute_long(
    "PC:A000",                                      // cpu
    "00A000:CF563412",                              // bus
    ("CMP", "$123456"),                             // expected
    0xA004,                                         // expected_pc
)]
#[case::absolute_long_indexed(
    "PC:A000",                                      // cpu
    "00A000:DF563412",                              // bus
    ("CMP", "$123456,X"),                           // expected
    0xA004,                                         // expected_pc
)]
#[case::direct(
    "PC:A000",                                      // cpu
    "00A000:C504",                                  // bus
    ("CMP", "$04"),                                 // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect_indexed(
    "PC:A000",                                      // cpu
    "00A000:D104",                                  // bus
    ("CMP", "($04),Y"),                             // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indexed_indirect(
    "PC:A000",                                      // cpu
    "00A000:C104",                                  // bus
    ("CMP", "($04,X)"),                             // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indexed_x(
    "PC:A000",                                      // cpu
    "00A000:D504",                                  // bus
    ("CMP", "$04,X"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect(
    "PC:A000",                                      // cpu
    "00A000:D244",                                  // bus
    ("CMP", "($44)"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect_long(
    "PC:A000",                                      // cpu
    "00A000:C744",                                  // bus
    ("CMP", "[$44]"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect_long_indexed(
    "PC:A000",                                      // cpu
    "00A000:D744",                                  // bus
    ("CMP", "[$44],Y"),                             // expected
    0xA002,                                         // expected_pc
)]
#[case::immediate(
    "PC:A000",                                      // cpu
    "00A000:C9FFFF",                                // bus
    ("CMP", "#$FFFF"),                              // expected
    0xA003,                                         // expected_pc
)]
#[case::stack_relative(
    "PC:A000",                                      // cpu
    "00A000:C304",                                  // bus
    ("CMP", "$04,S"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::stack_relative_indirect_indexed(
    "PC:A000",                                      // cpu
    "00A000:D304",                                  // bus
    ("CMP", "($04,S),Y"),                           // expected
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
