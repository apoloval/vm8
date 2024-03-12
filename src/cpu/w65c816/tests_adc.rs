use self::status::FlagExpectation;

use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case::emulation(
    "P.E:1,PC:A000,A:1123",                         // cpu
    "00A000:6945",                                  // bus
    0x1168,                                         // expected
    "C:0,Z:0,V:0,N:0",                              // expected_flags_set
)]
#[case::native_8bit(
    "P.E:0,P.M:1,PC:A000,A:1123",                   // cpu
    "00A000:6945",                                  // bus
    0x1168,                                         // expected
    "C:0,Z:0,V:0,N:0",                              // expected_flags_set
)]
#[case::native_8bit_bcd(
    "P.E:0,P.M:1,P.D:1,PC:A000,A:1123",             // cpu
    "00A000:6929",                                  // bus
    0x1152,                                         // expected
    "C:0,Z:0,V:0,N:0",                              // expected_flags_set
)]
#[case::native_8bit_carry_in(
    "P.E:0,P.M:1,P.C:1,PC:A000,A:1123",             // cpu
    "00A000:6945",                                  // bus
    0x1169,                                         // expected
    "C:0,Z:0,V:0,N:0",                              // expected_flags_set
)]
#[case::native_8bit_carry_out(
    "P.E:0,P.M:1,PC:A000,A:11F0",                   // cpu
    "00A000:6911",                                  // bus
    0x1101,                                         // expected
    "C:1,Z:0,V:0,N:0",                              // expected_flags_set
)]
#[case::native_8bit_zero(
    "P.E:0,P.M:1,PC:A000,A:1100",                   // cpu
    "00A000:6900",                                  // bus
    0x1100,                                         // expected
    "C:0,Z:1,V:0,N:0",                              // expected_flags_set
)]
#[case::native_8bit_overflow(
    "P.E:0,P.M:1,PC:A000,A:117F",                   // cpu
    "00A000:6901",                                  // bus
    0x1180,                                         // expected
    "C:0,Z:0,V:1,N:1",                              // expected_flags_set
)]
#[case::native_8bit_neg(
    "P.E:0,P.M:1,PC:A000,A:1182",                   // cpu
    "00A000:6901",                                  // bus
    0x1183,                                         // expected
    "C:0,Z:0,V:0,N:1",                              // expected_flags_set
)]
#[case::native_16bit(
    "P.E:0,P.M:0,PC:A000,A:1234",                   // cpu
    "00A000:697856",                                // bus
    0x68AC,                                         // expected
    "C:0,Z:0,V:0,N:0",                              // expected_flags_set
)]
#[case::native_16bit_bcd(
    "P.E:0,P.M:0,P.D:1,PC:A000,A:1234",             // cpu
    "00A000:697856",                                // bus
    0x6912,                                         // expected
    "C:0,Z:0,V:0,N:0",                              // expected_flags_set
)]
#[case::native_16bit_carry_in(
    "P.E:0,P.M:0,P.C:1,PC:A000,A:1234",             // cpu
    "00A000:697856",                                // bus
    0x68AD,                                         // expected
    "C:0,Z:0,V:0,N:0",                              // expected_flags_set
)]
#[case::native_16bit_carry_out(
    "P.E:0,P.M:0,PC:A000,A:FFF0",                   // cpu
    "00A000:691100",                                // bus
    0x0001,                                         // expected
    "C:1,Z:0,V:0,N:0",                              // expected_flags_set
)]
#[case::native_16bit_zero(
    "P.E:0,P.M:0,PC:A000,A:0000",                   // cpu
    "00A000:690000",                                // bus
    0x0000,                                         // expected
    "C:0,Z:1,V:0,N:0",                              // expected_flags_set
)]
#[case::native_16bit_overflow(
    "P.E:0,P.M:0,PC:A000,A:7FF0",                   // cpu
    "00A000:691100",                                // bus
    0x8001,                                         // expected
    "C:0,Z:0,V:1,N:1",                              // expected_flags_set
)]
#[case::native_16bit_neg(
    "P.E:0,P.M:0,PC:A000,A:8122",                   // cpu
    "00A000:690100",                                // bus
    0x8123,                                         // expected
    "C:0,Z:0,V:0,N:1",                              // expected_flags_set
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
    "00A000:6D5634",                                // bus
    ("ADC", "$3456"),                               // expected
    0xA003,                                         // expected_pc
)]
#[case::absolute_indexed_x(
    "PC:A000",                                      // cpu
    "00A000:7D5634",                                // bus
    ("ADC", "$3456,X"),                             // expected
    0xA003,                                         // expected_pc
)]
#[case::absolute_indexed_y(
    "PC:A000",                                      // cpu
    "00A000:795634",                                // bus
    ("ADC", "$3456,Y"),                             // expected
    0xA003,                                         // expected_pc
)]
#[case::absolute_long(
    "PC:A000",                                      // cpu
    "00A000:6F563412",                              // bus
    ("ADC", "$123456"),                             // expected
    0xA004,                                         // expected_pc
)]
#[case::absolute_long_indexed(
    "PC:A000",                                      // cpu
    "00A000:7F563412",                              // bus
    ("ADC", "$123456,X"),                           // expected
    0xA004,                                         // expected_pc
)]
#[case::direct(
    "PC:A000",                                      // cpu
    "00A000:6504",                                  // bus
    ("ADC", "$04"),                                 // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect_indexed(
    "PC:A000",                                      // cpu
    "00A000:7104",                                  // bus
    ("ADC", "($04),Y"),                             // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indexed_indirect(
    "PC:A000",                                      // cpu
    "00A000:6104",                                  // bus
    ("ADC", "($04,X)"),                             // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indexed_x(
    "PC:A000",                                      // cpu
    "00A000:7504",                                  // bus
    ("ADC", "$04,X"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect(
    "PC:A000",                                      // cpu
    "00A000:7244",                                  // bus
    ("ADC", "($44)"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect_long(
    "PC:A000",                                      // cpu
    "00A000:6744",                                  // bus
    ("ADC", "[$44]"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect_long_indexed(
    "PC:A000",                                      // cpu
    "00A000:7744",                                  // bus
    ("ADC", "[$44],Y"),                             // expected
    0xA002,                                         // expected_pc
)]
#[case::immediate(
    "PC:A000",                                      // cpu
    "00A000:69FFFF",                                // bus
    ("ADC", "#$FFFF"),                              // expected
    0xA003,                                         // expected_pc
)]
#[case::stack_relative(
    "PC:A000",                                      // cpu
    "00A000:6304",                                  // bus
    ("ADC", "$04,S"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::stack_relative_indirect_indexed(
    "PC:A000",                                      // cpu
    "00A000:7304",                                  // bus
    ("ADC", "($04,S),Y"),                           // expected
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
