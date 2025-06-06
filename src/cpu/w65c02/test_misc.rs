use crate::cpu::w65c02::{Bus, CPU, bus::FakeBus};

#[test]
fn test_nop() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0xEA); // NOP
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cycles, 2);
}
