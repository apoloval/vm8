use self::status::FlagExpectation;

use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case::emulation(
    "P.E:1,PC:A000,A:1123",                         // cpu
    "00A000:A945",                                  // bus
    0x1145,                                         // expected
    "Z:0,N:0",                                      // expected_flags_set
)]
#[case::emulation_negative(
    "P.E:1,PC:A000,A:1123",                         // cpu
    "00A000:A9AB",                                  // bus
    0x11AB,                                         // expected
    "Z:0,N:1",                                      // expected_flags_set
)]
#[case::emulation_zero(
    "P.E:1,PC:A000,A:1123",                         // cpu
    "00A000:A900",                                  // bus
    0x1100,                                         // expected
    "Z:1,N:0",                                      // expected_flags_set
)]
#[case::native_8bit(
    "P.E:0,P.M:1,PC:A000,A:1123",                   // cpu
    "00A000:A945",                                  // bus
    0x1145,                                         // expected
    "Z:0,N:0",                                      // expected_flags_set
)]
#[case::native_8bit_negative(
    "P.E:0,P.M:1,PC:A000,A:1123",                   // cpu
    "00A000:A9AB",                                  // bus
    0x11AB,                                         // expected
    "Z:0,N:1",                                      // expected_flags_set
)]
#[case::native_8bit_zero(
    "P.E:0,P.M:1,PC:A000,A:1123",                   // cpu
    "00A000:A900",                                  // bus
    0x1100,                                         // expected
    "Z:1,N:0",                                      // expected_flags_set
)]
#[case::native_16bit(
    "P.E:0,P.M:0,PC:A000,A:1123",                   // cpu
    "00A000:A96745",                                // bus
    0x4567,                                         // expected
    "Z:0,N:0",                                      // expected_flags_set
)]
#[case::native_16bit_negative(
    "P.E:0,P.M:0,PC:A000,A:1123",                   // cpu
    "00A000:A9CDAB",                                // bus
    0xABCD,                                         // expected
    "Z:0,N:1",                                      // expected_flags_set
)]
#[case::native_16bit_zero(
    "P.E:0,P.M:0,PC:A000,A:1123",                   // cpu
    "00A000:A90000",                                // bus
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
#[case::absolute(
    "PC:A000",                                      // cpu
    "00A000:ADCDAB",                                // bus
    ("LDA", "$ABCD"),                               // expected
    0xA003,                                         // expected_pc
)]
#[case::absolute_indexed_x(
    "PC:A000",                                      // cpu
    "00A000:BDCDAB",                                // bus
    ("LDA", "$ABCD,X"),                             // expected
    0xA003,                                         // expected_pc
)]
#[case::absolute_indexed_y(
    "PC:A000",                                      // cpu
    "00A000:B9CDAB",                                // bus
    ("LDA", "$ABCD,Y"),                             // expected
    0xA003,                                         // expected_pc
)]
#[case::absolute_long(
    "PC:A000",                                      // cpu
    "00A000:AFEFCDAB",                              // bus
    ("LDA", "$ABCDEF"),                             // expected
    0xA004,                                         // expected_pc
)]
#[case::absolute_long_indexed(
    "PC:A000",                                      // cpu
    "00A000:BFEFCDAB",                              // bus
    ("LDA", "$ABCDEF,X"),                           // expected
    0xA004,                                         // expected_pc
)]
#[case::direct(
    "PC:A000",                                      // cpu
    "00A000:A5AB",                                  // bus
    ("LDA", "$AB"),                                 // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indexed_indirect(
    "PC:A000",                                      // cpu
    "00A000:A1AB",                                  // bus
    ("LDA", "($AB,X)"),                             // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indexed_x(
    "PC:A000",                                      // cpu
    "00A000:B5AB",                                  // bus
    ("LDA", "$AB,X"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect(
    "PC:A000",                                      // cpu
    "00A000:B2AB",                                  // bus
    ("LDA", "($AB)"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect_indexed(
    "PC:A000",                                      // cpu
    "00A000:B1AB",                                  // bus
    ("LDA", "($AB),Y"),                             // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect_long(
    "PC:A000",                                      // cpu
    "00A000:A7AB",                                  // bus
    ("LDA", "[$AB]"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indirect_long_indexed(
    "PC:A000",                                      // cpu
    "00A000:B7AB",                                  // bus
    ("LDA", "[$AB],Y"),                             // expected
    0xA002,                                         // expected_pc
)]
#[case::immediate_8bit(
    "P.E:1,PC:A000",                                // cpu
    "00A000:A9AB",                                  // bus
    ("LDA", "#$00AB"),                              // expected
    0xA002,                                         // expected_pc
)]
#[case::immediate_16bit(
    "PC:A000",                                      // cpu
    "00A000:A9CDAB",                                // bus
    ("LDA", "#$ABCD"),                              // expected
    0xA003,                                         // expected_pc
)]
#[case::stack_relative(
    "PC:A000",                                      // cpu
    "00A000:A3AB",                                  // bus
    ("LDA", "$AB,S"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::stack_relative_indirect_indexed(
    "PC:A000",                                      // cpu
    "00A000:B3AB",                                  // bus
    ("LDA", "($AB,S),Y"),                           // expected
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
