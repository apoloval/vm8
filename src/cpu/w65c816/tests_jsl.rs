use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case::absolute_long(
    "PC:B000,PBR:A0,SP:FFFF",                       // cpu
    "A0B000:223412C0",                              // bus
    ("JSL", "$C01234"),                             // expected_inst
    0xA0,                                           // expected_pbr
    0x1234,                                         // expected_pc
    &[0x03, 0xB0, 0xA0],                            // expected_stack
)]
fn test_jsl(
    #[case] mut cpu: CPU,
    #[case] mut bus: bus::Fake,
    #[case] expected_inst: (&'static str, &'static str),
    #[case] expected_pbr: u8,
    #[case] expected_pc: u16,
    #[case] expected_stack: &[u8],
) {
    let mut reporter = ev::Retain::new();
    cpu.step(&mut bus, &mut reporter);

    let (expected_inst, expected_ops) = expected_inst;
    reporter.assert_exec(expected_inst, expected_ops);
    assert::program_counter(&cpu, expected_pbr, expected_pc);
    assert::stack_bytes(&cpu, &bus, 0, expected_stack);
}
