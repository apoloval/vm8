use super::{Addr, Bus, CPU};

pub fn stack_byte(cpu: &CPU, bus: &impl Bus, offset: u16, expected: u8) {
    assert_eq!(bus.read_byte(Addr::from(0, cpu.regs.sp()+offset)), expected);
}

pub fn program_counter(cpu: &CPU, expected_bank: u8, expected_pc: u16) {
    assert_eq!(cpu.regs.pbr(), expected_bank);
    assert_eq!(cpu.regs.pc(), expected_pc);
}

pub fn program_state(cpu: &CPU, expected: u8) {
    assert_eq!(cpu.regs.p(), expected);
}

pub fn accum(cpu: &CPU, expected: u16) {
    assert_eq!(cpu.regs.a(), expected);
}

pub fn index_x(cpu: &CPU, expected: u16) {
    assert_eq!(cpu.regs.x(), expected);
}

pub fn index_y(cpu: &CPU, expected: u16) {
    assert_eq!(cpu.regs.y(), expected);
}
