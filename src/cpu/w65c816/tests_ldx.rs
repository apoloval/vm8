use self::status::FlagExpectation;

use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case::emulation(
    "P.E:1,PC:A000,X:23",                           // cpu
    "00A000:A245",                                  // bus
    0x45,                                           // expected
    "Z:0,N:0",                                      // expected_flags_set
)]
#[case::emulation_negative(
    "P.E:1,PC:A000,X:23",                           // cpu
    "00A000:A2AB",                                  // bus
    0xAB,                                           // expected
    "Z:0,N:1",                                      // expected_flags_set
)]
#[case::emulation_zero(
    "P.E:1,PC:A000,X:23",                           // cpu
    "00A000:A200",                                  // bus
    0x00,                                           // expected
    "Z:1,N:0",                                      // expected_flags_set
)]
#[case::native_8bit(
    "P.E:0,P.X:1,PC:A000,X:23",                     // cpu
    "00A000:A245",                                  // bus
    0x45,                                           // expected
    "Z:0,N:0",                                      // expected_flags_set
)]
#[case::native_8bit_negative(
    "P.E:0,P.X:1,PC:A000,X:23",                     // cpu
    "00A000:A2AB",                                  // bus
    0xAB,                                           // expected
    "Z:0,N:1",                                      // expected_flags_set
)]
#[case::native_8bit_zero(
    "P.E:0,P.X:1,PC:A000,X:23",                     // cpu
    "00A000:A200",                                  // bus
    0x00,                                           // expected
    "Z:1,N:0",                                      // expected_flags_set
)]
#[case::native_16bit(
    "P.E:0,P.X:0,PC:A000,X:1123",                   // cpu
    "00A000:A26745",                                // bus
    0x4567,                                         // expected
    "Z:0,N:0",                                      // expected_flags_set
)]
#[case::native_16bit_negative(
    "P.E:0,P.X:0,PC:A000,X:1123",                   // cpu
    "00A000:A2CDAB",                                // bus
    0xABCD,                                         // expected
    "Z:0,N:1",                                      // expected_flags_set
)]
#[case::native_16bit_zero(
    "P.E:0,P.X:0,PC:A000,X:1123",                   // cpu
    "00A000:A20000",                                // bus
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
#[case::absolute(
    "PC:A000",                                      // cpu
    "00A000:AECDAB",                                // bus
    ("LDX", "$ABCD"),                               // expected
    0xA003,                                         // expected_pc
)]
#[case::absolute_indexed_y(
    "PC:A000",                                      // cpu
    "00A000:BECDAB",                                // bus
    ("LDX", "$ABCD,Y"),                             // expected
    0xA003,                                         // expected_pc
)]
#[case::direct(
    "PC:A000",                                      // cpu
    "00A000:A6AB",                                  // bus
    ("LDX", "$AB"),                                 // expected
    0xA002,                                         // expected_pc
)]
#[case::direct_indexed_y(
    "PC:A000",                                      // cpu
    "00A000:B6AB",                                  // bus
    ("LDX", "$AB,Y"),                               // expected
    0xA002,                                         // expected_pc
)]
#[case::immediate_8bit(
    "P.E:1,PC:A000",                                // cpu
    "00A000:A2AB",                                  // bus
    ("LDX", "#$00AB"),                              // expected
    0xA002,                                         // expected_pc
)]
#[case::immediate_16bit(
    "PC:A000",                                      // cpu
    "00A000:A2CDAB",                                // bus
    ("LDX", "#$ABCD"),                              // expected
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
