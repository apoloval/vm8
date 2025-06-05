use crate::cpu::w65c02::{Bus, CPU, bus::FakeBus, cpu::Flags};

#[test]
fn test_adc() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Immediate
    cpu.pc = 0x2000;
    cpu.a = 0x04;
    bus.mem_write(0x2000, 0x69); // ADC immediate
    bus.mem_write(0x2001, 0x03);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x07);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::OVERFLOW));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // With carry
    cpu.pc = 0x2000;
    cpu.a = 0x04;
    cpu.status.insert(Flags::CARRY);
    bus.mem_write(0x2000, 0x69); // ADC immediate
    bus.mem_write(0x2001, 0x03);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x08);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::OVERFLOW));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Overflow
    cpu.pc = 0x2000;
    cpu.a = 0x7F;
    bus.mem_write(0x2000, 0x69); // ADC immediate
    bus.mem_write(0x2001, 0x01);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x80);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(cpu.status.contains(Flags::OVERFLOW));
    assert!(cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Zero
    cpu.pc = 0x2000;
    cpu.a = 0x00;
    bus.mem_write(0x2000, 0x69); // ADC immediate
    bus.mem_write(0x2001, 0x00);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x00);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::OVERFLOW));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Carry
    cpu.pc = 0x2000;
    cpu.a = 0xFF;
    bus.mem_write(0x2000, 0x69); // ADC immediate
    bus.mem_write(0x2001, 0x01);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x00);
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::OVERFLOW));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);
}

#[test]
fn test_and() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Immediate
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    bus.mem_write(0x2000, 0x29); // AND immediate
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x42);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Zero
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    bus.mem_write(0x2000, 0x29); // AND immediate
    bus.mem_write(0x2001, 0x00);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x00);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Negative
    cpu.pc = 0x2000;
    cpu.a = 0x80;
    bus.mem_write(0x2000, 0x29); // AND immediate
    bus.mem_write(0x2001, 0x80);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x80);
    assert!(cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);
}

#[test]
fn test_eor() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Immediate
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    bus.mem_write(0x2000, 0x49); // EOR immediate
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x00);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Non-zero
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    bus.mem_write(0x2000, 0x49); // EOR immediate
    bus.mem_write(0x2001, 0x00);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x42);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Negative
    cpu.pc = 0x2000;
    cpu.a = 0x00;
    bus.mem_write(0x2000, 0x49); // EOR immediate
    bus.mem_write(0x2001, 0x80);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x80);
    assert!(cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);
}

#[test]
fn test_ora() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Immediate
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    bus.mem_write(0x2000, 0x09); // ORA immediate
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x42);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Zero
    cpu.pc = 0x2000;
    cpu.a = 0x00;
    bus.mem_write(0x2000, 0x09); // ORA immediate
    bus.mem_write(0x2001, 0x00);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x00);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Negative
    cpu.pc = 0x2000;
    cpu.a = 0x00;
    bus.mem_write(0x2000, 0x09); // ORA immediate
    bus.mem_write(0x2001, 0x80);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x80);
    assert!(cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);
}

#[test]
fn test_cmp() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Equal
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    bus.mem_write(0x2000, 0xC9); // CMP immediate
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Greater
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    bus.mem_write(0x2000, 0xC9); // CMP immediate
    bus.mem_write(0x2001, 0x41);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Less
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    bus.mem_write(0x2000, 0xC9); // CMP immediate
    bus.mem_write(0x2001, 0x43);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);
}

#[test]
fn test_cpx() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Equal
    cpu.pc = 0x2000;
    cpu.x = 0x42;
    bus.mem_write(0x2000, 0xE0); // CPX immediate
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Greater
    cpu.pc = 0x2000;
    cpu.x = 0x42;
    bus.mem_write(0x2000, 0xE0); // CPX immediate
    bus.mem_write(0x2001, 0x41);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Less
    cpu.pc = 0x2000;
    cpu.x = 0x42;
    bus.mem_write(0x2000, 0xE0); // CPX immediate
    bus.mem_write(0x2001, 0x43);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);
}

#[test]
fn test_cpy() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Equal
    cpu.pc = 0x2000;
    cpu.y = 0x42;
    bus.mem_write(0x2000, 0xC0); // CPY immediate
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Greater
    cpu.pc = 0x2000;
    cpu.y = 0x42;
    bus.mem_write(0x2000, 0xC0); // CPY immediate
    bus.mem_write(0x2001, 0x41);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Less
    cpu.pc = 0x2000;
    cpu.y = 0x42;
    bus.mem_write(0x2000, 0xC0); // CPY immediate
    bus.mem_write(0x2001, 0x43);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);
}

#[test]
fn test_sbc() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Immediate
    cpu.pc = 0x2000;
    cpu.a = 0x04;
    cpu.status.insert(Flags::CARRY);
    bus.mem_write(0x2000, 0xE9); // SBC immediate
    bus.mem_write(0x2001, 0x03);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x01);
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::OVERFLOW));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Without carry
    cpu.pc = 0x2000;
    cpu.a = 0x05;
    cpu.status.remove(Flags::CARRY);
    bus.mem_write(0x2000, 0xE9); // SBC immediate
    bus.mem_write(0x2001, 0x03);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x01);
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::OVERFLOW));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Overflow
    cpu.pc = 0x2000;
    cpu.a = 0x80;
    cpu.status.insert(Flags::CARRY);
    bus.mem_write(0x2000, 0xE9); // SBC immediate
    bus.mem_write(0x2001, 0x01);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x7F);
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(cpu.status.contains(Flags::OVERFLOW));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Zero
    cpu.pc = 0x2000;
    cpu.a = 0x00;
    cpu.status.insert(Flags::CARRY);
    bus.mem_write(0x2000, 0xE9); // SBC immediate
    bus.mem_write(0x2001, 0x00);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x00);
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::OVERFLOW));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Carry
    cpu.pc = 0x2000;
    cpu.a = 0x00;
    cpu.status.insert(Flags::CARRY);
    bus.mem_write(0x2000, 0xE9); // SBC immediate
    bus.mem_write(0x2001, 0x01);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0xFF);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::OVERFLOW));
    assert!(cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);
}

#[test]
fn test_inc() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Zero page
    cpu.pc = 0x0000;
    bus.mem_write(0x0000, 0xE6);
    bus.mem_write(0x0001, 0x42);
    bus.mem_write(0x0042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0002);
    assert_eq!(bus.mem_read(0x0042), 0x43);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert_eq!(cycles, 5);

    // Zero page X
    cpu.pc = 0x0000;
    cpu.x = 0x01;
    bus.mem_write(0x0000, 0xF6);
    bus.mem_write(0x0001, 0x41);
    bus.mem_write(0x0042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0002);
    assert_eq!(bus.mem_read(0x0042), 0x43);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert_eq!(cycles, 6);

    // Absolute
    cpu.pc = 0x0000;
    bus.mem_write(0x0000, 0xEE);
    bus.mem_write(0x0001, 0x42);
    bus.mem_write(0x0002, 0x00);
    bus.mem_write(0x0042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0003);
    assert_eq!(bus.mem_read(0x0042), 0x43);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert_eq!(cycles, 6);

    // Absolute X
    cpu.pc = 0x0000;
    cpu.x = 0x01;
    bus.mem_write(0x0000, 0xFE);
    bus.mem_write(0x0001, 0x41);
    bus.mem_write(0x0002, 0x00);
    bus.mem_write(0x0042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0003);
    assert_eq!(bus.mem_read(0x0042), 0x43);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert_eq!(cycles, 7);
}

#[test]
fn test_dec() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Zero page
    cpu.pc = 0x0000;
    bus.mem_write(0x0000, 0xC6);
    bus.mem_write(0x0001, 0x42);
    bus.mem_write(0x0042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0002);
    assert_eq!(bus.mem_read(0x0042), 0x41);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert_eq!(cycles, 5);

    // Zero page X
    cpu.pc = 0x0000;
    cpu.x = 0x01;
    bus.mem_write(0x0000, 0xD6);
    bus.mem_write(0x0001, 0x41);
    bus.mem_write(0x0042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0002);
    assert_eq!(bus.mem_read(0x0042), 0x41);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert_eq!(cycles, 6);

    // Absolute
    cpu.pc = 0x0000;
    bus.mem_write(0x0000, 0xCE);
    bus.mem_write(0x0001, 0x42);
    bus.mem_write(0x0002, 0x00);
    bus.mem_write(0x0042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0003);
    assert_eq!(bus.mem_read(0x0042), 0x41);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert_eq!(cycles, 6);

    // Absolute X
    cpu.pc = 0x0000;
    cpu.x = 0x01;
    bus.mem_write(0x0000, 0xDE);
    bus.mem_write(0x0001, 0x41);
    bus.mem_write(0x0002, 0x00);
    bus.mem_write(0x0042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0003);
    assert_eq!(bus.mem_read(0x0042), 0x41);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert_eq!(cycles, 7);
}

#[test]
fn test_inx() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Implied
    cpu.pc = 0x0000;
    bus.mem_write(0x0000, 0xE8);
    cpu.x = 0x42;
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0001);
    assert_eq!(cpu.x, 0x43);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert_eq!(cycles, 2);
}

#[test]
fn test_iny() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Implied
    cpu.pc = 0x0000;
    bus.mem_write(0x0000, 0xC8);
    cpu.y = 0x42;
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0001);
    assert_eq!(cpu.y, 0x43);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert_eq!(cycles, 2);
}

#[test]
fn test_dex() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Implied
    cpu.pc = 0x0000;
    bus.mem_write(0x0000, 0xCA);
    cpu.x = 0x42;
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0001);
    assert_eq!(cpu.x, 0x41);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert_eq!(cycles, 2);
}

#[test]
fn test_dey() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Implied
    cpu.pc = 0x0000;
    bus.mem_write(0x0000, 0x88);
    cpu.y = 0x42;
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0001);
    assert_eq!(cpu.y, 0x41);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert_eq!(cycles, 2);
}
