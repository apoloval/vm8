use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case::emulation(
    "P.E:1,PC:B000,PBR:00,SP:01FC",                 // cpu
    "00B000:40,0001FD:AB02B0",                      // bus
    ("RTI", ""),                                    // expected_inst
    0x00,                                           // expected_pbr
    0xB002,                                         // expected_pc
    0x01FF,                                         // expected_sp
    0xAB,                                           // expected_p
)]
#[case::native(
    "P.E:0,PC:B000,PBR:00,SP:FFFB",                 // cpu
    "00B000:40,00FFFC:AB02B040",                    // bus
    ("RTI", ""),                                    // expected_inst
    0x40,                                           // expected_pbr
    0xB002,                                         // expected_pc
    0xFFFF,                                         // expected_sp
    0xAB,                                           // expected_p
)]
fn test_rti(
    #[case] mut cpu: CPU,
    #[case] mut bus: bus::Fake,
    #[case] expected_inst: (&'static str, &'static str),
    #[case] expected_pbr: u8,
    #[case] expected_pc: u16,
    #[case] expected_sp: u16,
    #[case] expected_p: u8,
) {
    let mut reporter = ev::Retain::new();
    cpu.step(&mut bus, &mut reporter);

    let (expected_inst, expected_ops) = expected_inst;
    reporter.assert_exec(expected_inst, expected_ops);
    assert::program_counter(&cpu, expected_pbr, expected_pc);
    assert::stack_pointer(&cpu, expected_sp);
    assert::status(&cpu, expected_p);
}
