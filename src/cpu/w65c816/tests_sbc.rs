use self::status::FlagExpectation;

use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case::emulation(
    "P.E:1,P.C:1,PC:A000,A:117A",                   // cpu
    "00A000:E912",                                  // bus
    0x1168,                                         // expected
    "C:1,Z:0,V:0,N:0",                              // expected_flags_set
)]
#[case::native_8bit(
    "P.E:0,P.M:1,P.C:1,PC:A000,A:117A",             // cpu
    "00A000:E912",                                  // bus
    0x1168,                                         // expected
    "C:1,Z:0,V:0,N:0",                              // expected_flags_set
)]
#[case::native_8bit_bcd(
    "P.E:0,P.M:1,P.C:1,P.D:1,PC:A000,A:1143",       // cpu
    "00A000:E929",                                  // bus
    0x1114,                                         // expected
    "C:1,Z:0,V:0,N:0",                              // expected_flags_set
)]
#[case::native_8bit_borrow_in(
    "P.E:0,P.M:1,P.C:0,PC:A000,A:117A",             // cpu
    "00A000:E912",                                  // bus
    0x1167,                                         // expected
    "C:1,Z:0,V:0,N:0",                              // expected_flags_set
)]
#[case::native_8bit_borrow_out(
    "P.E:0,P.M:1,P.C:1,PC:A000,A:1102",             // cpu
    "00A000:E911",                                  // bus
    0x11F1,                                         // expected
    "C:0,Z:0,V:0,N:1",                              // expected_flags_set
)]
#[case::native_8bit_zero(
    "P.E:0,P.M:1,P.C:1,PC:A000,A:1142",             // cpu
    "00A000:E942",                                  // bus
    0x1100,                                         // expected
    "C:1,Z:1,V:0,N:0",                              // expected_flags_set
)]
#[case::native_8bit_overflow(
    "P.E:0,P.M:1,P.C:1,PC:A000,A:1182",             // cpu
    "00A000:E911",                                  // bus
    0x1171,                                         // expected
    "C:1,Z:0,V:1,N:0",                              // expected_flags_set
)]
#[case::native_8bit_neg(
    "P.E:0,P.M:1,P.C:1,PC:A000,A:1182",             // cpu
    "00A000:E901",                                  // bus
    0x1181,                                         // expected
    "C:1,Z:0,V:0,N:1",                              // expected_flags_set
)]
#[case::native_16bit(
    "P.E:0,P.M:0,P.C:1,PC:A000,A:5678",             // cpu
    "00A000:E93412",                                // bus
    0x4444,                                         // expected
    "C:1,Z:0,V:0,N:0",                              // expected_flags_set
)]
#[case::native_16bit_bcd(
    "P.E:0,P.M:0,P.C:1,P.D:1,PC:A000,A:9843",       // cpu
    "00A000:E92978",                                // bus
    0x2014,                                         // expected
    "C:1,Z:0,V:1,N:0",                              // expected_flags_set
)]
#[case::native_16bit_borrow_in(
    "P.E:0,P.M:0,P.C:0,PC:A000,A:5678",             // cpu
    "00A000:E93412",                                // bus
    0x4443,                                         // expected
    "C:1,Z:0,V:0,N:0",                              // expected_flags_set
)]
#[case::native_16bit_borrow_out(
    "P.E:0,P.M:0,P.C:1,PC:A000,A:1234",             // cpu
    "00A000:E97856",                                // bus
    0xBBBC,                                         // expected
    "C:0,Z:0,V:0,N:1",                              // expected_flags_set
)]
#[case::native_16bit_zero(
    "P.E:0,P.M:0,P.C:1,PC:A000,A:1234",             // cpu
    "00A000:E93412",                                // bus
    0x0000,                                         // expected
    "C:1,Z:1,V:0,N:0",                              // expected_flags_set
)]
#[case::native_16bit_overflow(
    "P.E:0,P.M:0,P.C:1,PC:A000,A:8123",             // cpu
    "00A000:E93412",                                // bus
    0x6EEF,                                         // expected
    "C:1,Z:0,V:1,N:0",                              // expected_flags_set
)]
#[case::native_16bit_neg(
    "P.E:0,P.M:0,P.C:1,PC:A000,A:ABCD",             // cpu
    "00A000:E93412",                                // bus
    0x9999,                                         // expected
    "C:1,Z:0,V:0,N:1",                              // expected_flags_set
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
    "00A000:ED5634",                                // bus
    ("SBC", "$3456"),                               // expected
    0xA003,                                         // expected_pc
)]
#[case::absolute_indexed_x(
    "PC:A000",                                      // cpu
    "00A000:FD5634",                                // bus
    ("SBC", "$3456,X"),                             // expected
    0xA003,                                         // expected_pc
)]
#[case::absolute_indexed_y(
    "PC:A000",                                      // cpu
    "00A000:F95634",                                // bus
    ("SBC", "$3456,Y"),                             // expected
    0xA003,                                         // expected_pc
)]
#[case::absolute_long(
    "PC:A000",                                      // cpu
    "00A000:EF563412",                              // bus
    ("SBC", "$123456"),                             // expected
    0xA004,                                         // expected_pc
)]
#[case::absolute_long_indexed(
    "PC:A000",                                      // cpu
    "00A000:FF563412",                              // bus
    ("SBC", "$123456,X"),                           // expected
    0xA004,                                         // expected_pc
)]
#[case::direct(
    "PC:A000",                                      // cpu
    "00A000:E504",                                  // bus
    ("SBC", "$04"),                                 // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect_indexed(
    "PC:A000",                                      // cpu
    "00A000:F104",                                  // bus
    ("SBC", "($04),Y"),                             // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indexed_indirect(
    "PC:A000",                                      // cpu
    "00A000:E104",                                  // bus
    ("SBC", "($04,X)"),                             // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indexed_x(
    "PC:A000",                                      // cpu
    "00A000:F504",                                  // bus
    ("SBC", "$04,X"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect(
    "PC:A000",                                      // cpu
    "00A000:F244",                                  // bus
    ("SBC", "($44)"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect_long(
    "PC:A000",                                      // cpu
    "00A000:E744",                                  // bus
    ("SBC", "[$44]"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect_long_indexed(
    "PC:A000",                                      // cpu
    "00A000:F744",                                  // bus
    ("SBC", "[$44],Y"),                             // expected
    0xA002,                                         // expected_pc
)]
#[case::immediate(
    "PC:A000",                                      // cpu
    "00A000:E9FFFF",                                // bus
    ("SBC", "#$FFFF"),                              // expected
    0xA003,                                         // expected_pc
)]
#[case::stack_relative(
    "PC:A000",                                      // cpu
    "00A000:E304",                                  // bus
    ("SBC", "$04,S"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::stack_relative_indirect_indexed(
    "PC:A000",                                      // cpu
    "00A000:F304",                                  // bus
    ("SBC", "($04,S),Y"),                           // expected
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
