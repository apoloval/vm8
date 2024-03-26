use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case::absolute(
    "PC:B000,PBR:A0",                               // cpu
    "A0B000:4C3412",                                // bus
    ("JMP", "$1234"),                               // expected_inst
    0xA0,                                           // expected_pbr
    0x1234,                                         // expected_pc
)]
#[case::absolute_long(
    "PC:B000,PBR:A0",                               // cpu
    "A0B000:5C3412C0",                              // bus
    ("JMP", "$C01234"),                             // expected_inst
    0xC0,                                           // expected_pbr
    0x1234,                                         // expected_pc
)]
#[case::absolute_indirect(
    "PC:B000,PBR:A0",                               // cpu
    "A0B000:6C3412,001234:7856",                    // bus
    ("JMP", "($1234)"),                             // expected_inst
    0xA0,                                           // expected_pbr
    0x5678,                                         // expected_pc
)]
#[case::absolute_indexed_indirect(
    "PC:B000,PBR:A0,X:02",                          // cpu
    "A0B000:7C3412,001236:7856",                    // bus
    ("JMP", "($1234,X)"),                           // expected_inst
    0xA0,                                           // expected_pbr
    0x5678,                                         // expected_pc
)]
#[case::absolute_indirect_long(
    "PC:B000,PBR:A0",                               // cpu
    "A0B000:DC3412,001234:7856B0",                  // bus
    ("JMP", "[$1234]"),                             // expected_inst
    0xB0,                                           // expected_pbr
    0x5678,                                         // expected_pc
)]
fn test_jmp(
    #[case] mut cpu: CPU,
    #[case] mut bus: bus::Fake,
    #[case] expected_inst: (&'static str, &'static str),
    #[case] expected_pbr: u8,
    #[case] expected_pc: u16,
) {
    let mut reporter = ev::Retain::new();
    cpu.step(&mut bus, &mut reporter);

    let (expected_inst, expected_ops) = expected_inst;
    reporter.assert_exec(expected_inst, expected_ops);
    assert::program_counter(&cpu, expected_pbr, expected_pc);
}
