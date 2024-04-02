use self::status::FlagExpectation;

use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case::carry(
    "PC:A000,P:FF",                                 // cpu
    "00A000:18",                                    // bus
    "CLC",                                          // expected_inst
    0xA001,                                         // expected_pc
    "C:0",                                          // expected_flags
)]
#[case::decimal(
    "PC:A000,P:FF",                                 // cpu
    "00A000:D8",                                    // bus
    "CLD",                                          // expected_inst
    0xA001,                                         // expected_pc
    "D:0",                                          // expected_flags
)]
#[case::interrupt(
    "PC:A000,P:FF",                                 // cpu
    "00A000:58",                                    // bus
    "CLI",                                          // expected_inst
    0xA001,                                         // expected_pc
    "I:0",                                          // expected_flags
)]
#[case::overflow(
    "PC:A000,P:FF",                                 // cpu
    "00A000:B8",                                    // bus
    "CLV",                                          // expected_inst
    0xA001,                                         // expected_pc
    "V:0",                                          // expected_flags
)]
fn test_clear(
    #[case] mut cpu: CPU,
    #[case] mut bus: bus::Fake,
    #[case] expected_inst: &'static str,
    #[case] expected_pc: u16,
    #[case] expected_flags: FlagExpectation,
) {
    let mut reporter = ev::Retain::new();
    cpu.step(&mut bus, &mut reporter);

    reporter.assert_exec(expected_inst, "");
    assert::program_counter(&cpu, cpu.regs.pbr(), expected_pc);
    expected_flags.assert(cpu.regs.p());
}
