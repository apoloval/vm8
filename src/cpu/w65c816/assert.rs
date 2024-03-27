use super::{Addr, Bus, CPU};

pub fn stack_byte(cpu: &CPU, bus: &impl Bus, offset: u16, expected: u8) {
    assert_eq!(
        bus.read_byte(Addr::from(0, cpu.regs.sp()+offset+1)), 
        expected, 
        "unexpected stack byte at {:#X}", 
        cpu.regs.sp()+offset+1,
    );
}

pub fn stack_bytes(cpu: &CPU, bus: &impl Bus, offset: u16, expected: &[u8]) {
    for (i, &expected) in expected.iter().enumerate() {
        stack_byte(cpu, bus, offset+i as u16, expected);
    }
}

pub fn program_counter(cpu: &CPU, expected_bank: u8, expected_pc: u16) {
    assert_eq!(cpu.regs.pbr(), expected_bank, "unexpected program bank: {:#X}", cpu.regs.pbr());
    assert_eq!(cpu.regs.pc(), expected_pc, "unexpected program counter: {:#X}", cpu.regs.pc());
}

pub fn program_state(cpu: &CPU, expected: u8) {
    assert_eq!(cpu.regs.p(), expected, "unexpected program state: {:#X}", cpu.regs.p());
}

pub fn accum(cpu: &CPU, expected: u16) {
    assert_eq!(cpu.regs.a(), expected, "unexpected accumulator: {:#X}", cpu.regs.a());
}

pub fn index_x(cpu: &CPU, expected: u16) {
    assert_eq!(cpu.regs.x(), expected, "unexpected index x: {:#X}", cpu.regs.x());
}

pub fn index_y(cpu: &CPU, expected: u16) {
    assert_eq!(cpu.regs.y(), expected, "unexpected index y: {:#X}", cpu.regs.y());
}
