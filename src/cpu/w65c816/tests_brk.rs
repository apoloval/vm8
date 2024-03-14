use super::*;
use crate::cpu::w65c816::assert;

use rstest::*;
    
#[rstest]
#[case::emulation(
    "P.E:1,PC:A000,P:AA,SP:FF",             // cpu
    "00A000:0000,00FFFE:3412",              // bus
    vec![0xBA, 0x02, 0xA0],                 // expected_stack
    0x1234,                                 // expected_pc        
    0xBA,                                   // expected_state
)]
#[case::native(
    "P.E:0,PBR:B0,PC:A000,P:AA,SP:E0FF",    // cpu
    "B0A000:0000,00FFE6:3412",              // bus
    vec![0xAA, 0x02, 0xA0, 0xB0],           // expected_stack
    0x1234,                                 // expected_pc        
    0xAA,                                   // expected_state
)]
fn test_brk(
    #[case] mut cpu: CPU, 
    #[case] mut bus: bus::Fake,
    #[case] expected_stack: Vec<u8>,
    #[case] expected_pc: u16,
    #[case] expected_state: u8,
) {
    cpu.step(&mut bus, &mut NullReporter);

    for (offset, expected) in expected_stack.iter().enumerate() {
        assert::stack_byte(&cpu, &bus, offset as u16+1, *expected);
    }
    assert::program_counter(&cpu, 0, expected_pc);
    assert::program_state(&cpu, expected_state);
}