use self::status::FlagExpectation;

use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case::emulation(
    "P.E:1,DP:4000,PC:A000,A:110F",                 // cpu
    "004012:55,00A000:0412",                        // bus
    0x12,                                           // expected_dir
    0x5F,                                           // expected
    "Z:0",                                          // expected_flags_set
)]
#[case::native_8bit(
    "P.E:0,P.M:1,DP:4000,PC:A000,A:110F",           // cpu
    "004012:55,00A000:0412",                        // bus
    0x12,                                           // expected_dir
    0x5F,                                           // expected
    "Z:0",                                          // expected_flags_set
)]
#[case::native_8bit_zero(
    "P.E:0,P.M:1,DP:4000,PC:A000,A:110F",           // cpu
    "004012:F0,00A000:0412",                        // bus
    0x12,                                           // expected_dir
    0xFF,                                           // expected
    "Z:1",                                          // expected_flags_set
)]
#[case::native_16bit(
    "P.E:0,P.M:0,DP:4000,PC:A000,A:0F0F",           // cpu
    "004012:5555,00A000:0412",                      // bus
    0x12,                                           // expected_dir
    0x5F5F,                                         // expected
    "Z:0",                                          // expected_flags_set
)]
#[case::native_16bit_zero(
    "P.E:0,P.M:0,DP:4000,PC:A000,A:0F0F",           // cpu
    "004012:F0F0,00A000:0412",                      // bus
    0x12,                                           // expected_dir
    0xFFFF,                                         // expected
    "Z:1",                                          // expected_flags_set
)]
fn test_results(
    #[case] mut cpu: CPU,
    #[case] mut bus: bus::Fake,
    #[case] expected_dir: u8,
    #[case] expected: u16,
    #[case] expected_flags: FlagExpectation,
) {
    cpu.step(&mut bus, &mut NullReporter);

    assert_eq!(cpu.read_direct_word(&mut bus, expected_dir, 0), expected);
    expected_flags.assert(cpu.regs.p());
}

#[rstest]
#[case::absolute(
    "PC:A000",                                      // cpu
    "00A000:0C5634",                                // bus
    ("TSB", "$3456"),                               // expected
    0xA003,                                         // expected_pc
)]
#[case::direct(
    "PC:A000",                                      // cpu
    "00A000:0404",                                  // bus
    ("TSB", "$04"),                                 // expected
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
