use crate::cpu::w65c02::{Bus, CPU, bus::FakeBus, cpu::Flags};

#[test]
fn test_jmp() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Absolute
    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0x4C); // JMP absolute
    bus.mem_write(0x2001, 0x34);
    bus.mem_write(0x2002, 0x12);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x1234);
    assert_eq!(cycles, 3);

    // Indirect
    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0x6C); // JMP indirect
    bus.mem_write(0x2001, 0x34);
    bus.mem_write(0x2002, 0x12);
    bus.mem_write(0x1234, 0x78);
    bus.mem_write(0x1235, 0x56);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x5678);
    assert_eq!(cycles, 5);
}

#[test]
fn test_jsr() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.sp = 0xFF;
    bus.mem_write(0x2000, 0x20); // JSR absolute
    bus.mem_write(0x2001, 0x34);
    bus.mem_write(0x2002, 0x12);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x1234);
    assert_eq!(cpu.sp, 0xFD);
    assert_eq!(bus.mem_read(0x01FF), 0x20);
    assert_eq!(bus.mem_read(0x01FE), 0x02);
    assert_eq!(cycles, 6);
}

#[test]
fn test_rts() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.sp = 0xFD;
    bus.mem_write(0x01FF, 0x20);
    bus.mem_write(0x01FE, 0x02);
    bus.mem_write(0x2000, 0x60); // RTS
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cpu.sp, 0xFF);
    assert_eq!(cycles, 6);
}

#[test]
fn test_rti() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.sp = 0xFC;
    bus.mem_write(0x01FF, 0x20);
    bus.mem_write(0x01FE, 0x02);
    bus.mem_write(0x01FD, 0x00);
    bus.mem_write(0x2000, 0x40); // RTI
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cpu.sp, 0xFF);
    assert_eq!(cpu.status.bits(), 0x00);
    assert_eq!(cycles, 6);
}

#[test]
fn test_brk() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.sp = 0xFF;
    bus.mem_write(0x2000, 0x00); // BRK
    bus.mem_write(0xFFFE, 0x34);
    bus.mem_write(0xFFFF, 0x12);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x1234);
    assert_eq!(cpu.sp, 0xFC);
    assert!(cpu.status.contains(Flags::INTERRUPT));
    assert_eq!(bus.mem_read(0x01FF), 0x20);
    assert_eq!(bus.mem_read(0x01FE), 0x02);
    assert!(Flags::from_bits(bus.mem_read(0x01FD)).unwrap().contains(Flags::BREAK));
    assert_eq!(cycles, 7);
} 