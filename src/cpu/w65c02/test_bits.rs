use crate::cpu::w65c02::{CPU, FakeBus, cpu::Flags, bus::Bus};

#[test]
fn test_bit() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Zero page
    cpu.pc = 0x0000;
    bus.mem_write(0x0000, 0x24);
    bus.mem_write(0x0001, 0x42);
    bus.mem_write(0x0042, 0xAA);
    cpu.a = 0x55;
    let inst = cpu.exec(&mut bus);
    assert!(cpu.status.contains(Flags::ZERO));
    assert!(cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::OVERFLOW)); 
    assert_eq!(inst.cycles, 3);

    // Absolute
    cpu.pc = 0x0000;
    bus.mem_write(0x0000, 0x2C);
    bus.mem_write(0x0001, 0x42);
    bus.mem_write(0x0002, 0x00);
    bus.mem_write(0x0042, 0x00);
    cpu.a = 0xFF;
    let inst = cpu.exec(&mut bus);
    assert!(cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::OVERFLOW));
    assert_eq!(inst.cycles, 4);
}
