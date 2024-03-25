use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;

#[rstest]
#[case::bcc_no_branch(
    "PC:A000,P.C:1",                                // cpu
    "00A000:9042",                                  // bus
    ("BCC", "$A044"),                               // expected_inst
    0xA002,                                         // expected_pc
)]
#[case::bcc_branch(
    "PC:A000,P.C:0",                                // cpu
    "00A000:9042",                                  // bus
    ("BCC", "$A044"),                               // expected_inst
    0xA044,                                         // expected_pc
)]
#[case::bcs_no_branch(
    "PC:A000,P.C:0",                                // cpu
    "00A000:B042",                                  // bus
    ("BCS", "$A044"),                               // expected_inst
    0xA002,                                         // expected_pc
)]
#[case::bcs_branch(
    "PC:A000,P.C:1",                                // cpu
    "00A000:B042",                                  // bus
    ("BCS", "$A044"),                               // expected_inst
    0xA044,                                         // expected_pc
)]
#[case::beq_no_branch(
    "PC:A000,P.Z:0",                                // cpu
    "00A000:F042",                                  // bus
    ("BEQ", "$A044"),                               // expected_inst
    0xA002,                                         // expected_pc
)]
#[case::beq_branch(
    "PC:A000,P.Z:1",                                // cpu
    "00A000:F042",                                  // bus
    ("BEQ", "$A044"),                               // expected_inst
    0xA044,                                         // expected_pc
)]
#[case::bmi_no_branch(
    "PC:A000,P.N:0",                                // cpu
    "00A000:3042",                                  // bus
    ("BMI", "$A044"),                               // expected_inst
    0xA002,                                         // expected_pc
)]
#[case::bmi_branch(
    "PC:A000,P.N:1",                                // cpu
    "00A000:3042",                                  // bus
    ("BMI", "$A044"),                               // expected_inst
    0xA044,                                         // expected_pc
)]
#[case::bne_no_branch(
    "PC:A000,P.Z:1",                                // cpu
    "00A000:D042",                                  // bus
    ("BNE", "$A044"),                               // expected_inst
    0xA002,                                         // expected_pc
)]
#[case::bne_branch(
    "PC:A000,P.Z:0",                                // cpu
    "00A000:D042",                                  // bus
    ("BNE", "$A044"),                               // expected_inst
    0xA044,                                         // expected_pc
)]
#[case::bpl_no_branch(
    "PC:A000,P.N:1",                                // cpu
    "00A000:1042",                                  // bus
    ("BPL", "$A044"),                               // expected_inst
    0xA002,                                         // expected_pc
)]
#[case::bpl_branch(
    "PC:A000,P.N:0",                                // cpu
    "00A000:1042",                                  // bus
    ("BPL", "$A044"),                               // expected_inst
    0xA044,                                         // expected_pc
)]
#[case::bra_positive(
    "PC:A000",                                      // cpu
    "00A000:8042",                                  // bus
    ("BRA", "$A044"),                               // expected_inst
    0xA044,                                         // expected_pc
)]
#[case::bra_negative(
    "PC:A000",                                      // cpu
    "00A000:80FE",                                  // bus
    ("BRA", "$A000"),                               // expected_inst
    0xA000,                                         // expected_pc
)]
#[case::brl_positive(
    "PC:A000",                                      // cpu
    "00A000:824210",                                  // bus
    ("BRL", "$B045"),                               // expected_inst
    0xB045,                                         // expected_pc
)]
#[case::brl_negative(
    "PC:A000",                                      // cpu
    "00A000:82FDFF",                                  // bus
    ("BRL", "$A000"),                               // expected_inst
    0xA000,                                         // expected_pc
)]
#[case::bvc_no_branch(
    "PC:A000,P.V:1",                                // cpu
    "00A000:5042",                                  // bus
    ("BVC", "$A044"),                               // expected_inst
    0xA002,                                         // expected_pc
)]
#[case::bvc_branch(
    "PC:A000,P.V:0",                                // cpu
    "00A000:5042",                                  // bus
    ("BVC", "$A044"),                               // expected_inst
    0xA044,                                         // expected_pc
)]
#[case::bvs_no_branch(
    "PC:A000,P.V:0",                                // cpu
    "00A000:7042",                                  // bus
    ("BVS", "$A044"),                               // expected_inst
    0xA002,                                         // expected_pc
)]
#[case::bvs_branch(
    "PC:A000,P.V:1",                                // cpu
    "00A000:7042",                                  // bus
    ("BVS", "$A044"),                               // expected_inst
    0xA044,                                         // expected_pc
)]
fn test_branch(
    #[case] mut cpu: CPU,
    #[case] mut bus: bus::Fake,
    #[case] expected_inst: (&'static str, &'static str),
    #[case] expected_pc: u16,
) {
    let mut reporter = ev::Retain::new();
    cpu.step(&mut bus, &mut reporter);

    let (expected_inst, expected_ops) = expected_inst;
    reporter.assert_exec(expected_inst, expected_ops);
    assert::program_counter(&cpu, cpu.regs.pbr(), expected_pc);
}
