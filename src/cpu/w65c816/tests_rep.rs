use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case::emulated_even_bits(
    "P.E:1,PC:A000,P:FF",                           // cpu
    "00A000:C2AA",                                  // bus
    ("REP", "#$AA"),                                // expected
    0xA002,                                         // expected_pc
    0x75,                                           // expected_flags
)]
#[case::native_even_bits(
    "P.E:0,PC:A000,P:FF",                           // cpu
    "00A000:C2AA",                                  // bus
    ("REP", "#$AA"),                                // expected
    0xA002,                                         // expected_pc
    0x55,                                           // expected_flags
)]
#[case::emulated_odd_bits(
    "P.E:1,PC:A000,P:FF",                           // cpu
    "00A000:C255",                                  // bus
    ("REP", "#$55"),                                // expected
    0xA002,                                         // expected_pc
    0xBA,                                           // expected_flags
)]
#[case::native_odd_bits(
    "P.E:0,PC:A000,P:FF",                           // cpu
    "00A000:C255",                                  // bus
    ("REP", "#$55"),                                // expected
    0xA002,                                         // expected_pc
    0xAA,                                           // expected_flags
)]
fn test_rep(
    #[case] mut cpu: CPU,
    #[case] mut bus: bus::Fake,
    #[case] expected: (&'static str, &'static str),
    #[case] expected_pc: u16,
    #[case] expected_flags: u8,
) {
    let mut reporter = ev::Retain::new();
    cpu.step(&mut bus, &mut reporter);

    let (expected_inst, expected_ops) = expected;
    reporter.assert_exec(expected_inst, expected_ops);
    assert::program_counter(&cpu, cpu.regs.pbr(), expected_pc);
    assert::status(&mut cpu, expected_flags);
}
