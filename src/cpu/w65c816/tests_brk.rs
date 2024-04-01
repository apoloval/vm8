use super::*;
use super::status::FlagExpectation;
use crate::cpu::w65c816::assert;

use rstest::*;
    
#[rstest]
#[case::emulation(
    "P.E:1,PC:A000,P:AA,SP:FF",             // cpu
    "00A000:0000,00FFFE:3412",              // bus
    &[0xBA, 0x02, 0xA0],                    // expected_stack
    0x1234,                                 // expected_pc        
    "I:1,D:0,B:1",                          // expected_flags
)]
#[case::native(
    "P.E:0,PBR:B0,PC:A000,P:AA,SP:E0FF",    // cpu
    "B0A000:0000,00FFE6:3412",              // bus
    &[0xAA, 0x02, 0xA0, 0xB0],              // expected_stack
    0x1234,                                 // expected_pc        
    "I:1,D:0",                              // expected_flags
)]
fn test_brk(
    #[case] mut cpu: CPU, 
    #[case] mut bus: bus::Fake,
    #[case] expected_stack: &[u8],
    #[case] expected_pc: u16,
    #[case] expected_flags: FlagExpectation,
) {
    cpu.step(&mut bus, &mut NullReporter);

    assert::stack_bytes(&cpu, &bus, 0, &expected_stack[..]);
    assert::program_counter(&cpu, 0, expected_pc);
    expected_flags.assert(cpu.regs.p());
}