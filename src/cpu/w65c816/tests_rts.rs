use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case(
    "PC:B000,PBR:A0,SP:FFFD",                       // cpu
    "A0B000:60,00FFFE:02B0",                        // bus
    ("RTS", ""),                                    // expected_inst
    0xA0,                                           // expected_pbr
    0xB003,                                         // expected_pc
    0xFFFF,                                         // expected_sp
)]
fn test_rts(
    #[case] mut cpu: CPU,
    #[case] mut bus: bus::Fake,
    #[case] expected_inst: (&'static str, &'static str),
    #[case] expected_pbr: u8,
    #[case] expected_pc: u16,
    #[case] expected_sp: u16,
) {
    let mut reporter = ev::Retain::new();
    cpu.step(&mut bus, &mut reporter);

    let (expected_inst, expected_ops) = expected_inst;
    reporter.assert_exec(expected_inst, expected_ops);
    assert::program_counter(&cpu, expected_pbr, expected_pc);
    assert::stack_pointer(&cpu, expected_sp);
}
