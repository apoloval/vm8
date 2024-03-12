use self::status::FlagExpectation;

use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case::emulation(
    "P.E:1,PC:A000,A:1122",                         // cpu
    "00A000:490F",                                  // bus
    0x112D,                                         // expected
    "Z:0,N:0",                                      // expected_flags_set
)]
#[case::native_8bit(
    "P.E:0,P.M:1,PC:A000,A:1122",                   // cpu
    "00A000:490F",                                  // bus
    0x112D,                                         // expected
    "Z:0,N:0",                                      // expected_flags_set
)]
#[case::native_16bit(
    "P.E:0,P.M:0,PC:A000,A:1122",                   // cpu
    "00A000:49F00F",                                // bus
    0x1ED2,                                         // expected
    "Z:0,N:0",                                      // expected_flags_set
)]
#[case::native_8bit_zero(
    "P.E:0,P.M:1,PC:A000,A:11FF",                   // cpu
    "00A000:49FF",                                  // bus
    0x1100,                                         // expected
    "Z:1,N:0",                                      // expected_flags_set
)]
#[case::native_16bit_zero(
    "P.E:0,P.M:0,PC:A000,A:FFFF",                   // cpu
    "00A000:49FFFF",                                // bus
    0x0000,                                         // expected
    "Z:1,N:0",                                      // expected_flags_set
)]
#[case::native_8bit_neg(
    "P.E:0,P.M:1,PC:A000,A:1122",                   // cpu
    "00A000:49F0",                                  // bus
    0x11D2,                                         // expected
    "Z:0,N:1",                                      // expected_flags_set
)]
#[case::native_16bit_neg(
    "P.E:0,P.M:0,PC:A000,A:1122",                   // cpu
    "00A000:49F0F0",                                // bus
    0xE1D2,                                         // expected
    "Z:0,N:1",                                      // expected_flags_set
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
    "00A000:4D5634",                                // bus
    ("EOR", "$3456"),                               // expected
    0xA003,                                         // expected_pc
)]
#[case::absolute_indexed_x(
    "PC:A000",                                      // cpu
    "00A000:5D5634",                                // bus
    ("EOR", "$3456,X"),                             // expected
    0xA003,                                         // expected_pc
)]
#[case::absolute_indexed_y(
    "PC:A000",                                      // cpu
    "00A000:595634",                                // bus
    ("EOR", "$3456,Y"),                             // expected
    0xA003,                                         // expected_pc
)]
#[case::absolute_long(
    "PC:A000",                                      // cpu
    "00A000:4F563412",                              // bus
    ("EOR", "$123456"),                             // expected
    0xA004,                                         // expected_pc
)]
#[case::absolute_long_indexed(
    "PC:A000",                                      // cpu
    "00A000:5F563412",                              // bus
    ("EOR", "$123456,X"),                           // expected
    0xA004,                                         // expected_pc
)]
#[case::direct(
    "PC:A000",                                      // cpu
    "00A000:4504",                                  // bus
    ("EOR", "$04"),                                 // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect_indexed(
    "PC:A000",                                      // cpu
    "00A000:5104",                                  // bus
    ("EOR", "($04),Y"),                             // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indexed_indirect(
    "PC:A000",                                      // cpu
    "00A000:4104",                                  // bus
    ("EOR", "($04,X)"),                             // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indexed_x(
    "PC:A000",                                      // cpu
    "00A000:5504",                                  // bus
    ("EOR", "$04,X"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect(
    "PC:A000",                                      // cpu
    "00A000:5244",                                  // bus
    ("EOR", "($44)"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect_long(
    "PC:A000",                                      // cpu
    "00A000:4744",                                  // bus
    ("EOR", "[$44]"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect_long_indexed(
    "PC:A000",                                      // cpu
    "00A000:5744",                                  // bus
    ("EOR", "[$44],Y"),                             // expected
    0xA002,                                         // expected_pc
)]
#[case::immediate(
    "PC:A000",                                      // cpu
    "00A000:49FFFF",                                // bus
    ("EOR", "#$FFFF"),                              // expected
    0xA003,                                         // expected_pc
)]
#[case::stack_relative(
    "PC:A000",                                      // cpu
    "00A000:4304",                                  // bus
    ("EOR", "$04,S"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::stack_relative_indirect_indexed(
    "PC:A000",                                      // cpu
    "00A000:5304",                                  // bus
    ("EOR", "($04,S),Y"),                           // expected
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
