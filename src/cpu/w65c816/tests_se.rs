use self::status::FlagExpectation;

use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case::carry(
    "PC:A000,P:00",                                 // cpu
    "00A000:38",                                    // bus
    "SEC",                                          // expected_inst
    0xA001,                                         // expected_pc
    "C:1",                                          // expected_flags
)]
#[case::decimal(
    "PC:A000,P:00",                                 // cpu
    "00A000:F8",                                    // bus
    "SED",                                          // expected_inst
    0xA001,                                         // expected_pc
    "D:1",                                          // expected_flags
)]
#[case::interrupt(
    "PC:A000,P:00",                                 // cpu
    "00A000:78",                                    // bus
    "SEI",                                          // expected_inst
    0xA001,                                         // expected_pc
    "I:1",                                          // expected_flags
)]
fn test_set(
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
