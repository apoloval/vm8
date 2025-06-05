use crate::cpu::w65c02::{Bus, CPU, bus::FakeBus, cpu::Flags};

#[test]
fn test_asl() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Accumulator
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    bus.mem_write(0x2000, 0x0A); // ASL A
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.a, 0x42);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Zero page
    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0x06); // ASL zero page
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x42, 0x21);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(bus.mem_read(0x42), 0x42);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 5);

    // Zero page X
    cpu.pc = 0x2000;
    cpu.x = 0x01;
    bus.mem_write(0x2000, 0x16); // ASL zero page,X
    bus.mem_write(0x2001, 0x41);
    bus.mem_write(0x42, 0x21);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(bus.mem_read(0x42), 0x42);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 6);

    // Absolute
    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0x0E); // ASL absolute
    bus.mem_write(0x2001, 0x34);
    bus.mem_write(0x2002, 0x12);
    bus.mem_write(0x1234, 0x21);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(bus.mem_read(0x1234), 0x42);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 6);

    // Absolute X
    cpu.pc = 0x2000;
    cpu.x = 0x01;
    bus.mem_write(0x2000, 0x1E); // ASL absolute,X
    bus.mem_write(0x2001, 0x33);
    bus.mem_write(0x2002, 0x12);
    bus.mem_write(0x1234, 0x21);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(bus.mem_read(0x1234), 0x42);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 7);

    // Carry
    cpu.pc = 0x2000;
    cpu.a = 0x80;
    bus.mem_write(0x2000, 0x0A); // ASL A
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.a, 0x00);
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Negative
    cpu.pc = 0x2000;
    cpu.a = 0x40;
    bus.mem_write(0x2000, 0x0A); // ASL A
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.a, 0x80);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);
}

#[test]
fn test_lsr() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Accumulator
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    bus.mem_write(0x2000, 0x4A); // LSR A
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.a, 0x21);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Zero page
    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0x46); // LSR zero page
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x42, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(bus.mem_read(0x42), 0x21);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 5);

    // Zero page X
    cpu.pc = 0x2000;
    cpu.x = 0x01;
    bus.mem_write(0x2000, 0x56); // LSR zero page,X
    bus.mem_write(0x2001, 0x41);
    bus.mem_write(0x42, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(bus.mem_read(0x42), 0x21);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 6);

    // Absolute
    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0x4E); // LSR absolute
    bus.mem_write(0x2001, 0x34);
    bus.mem_write(0x2002, 0x12);
    bus.mem_write(0x1234, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(bus.mem_read(0x1234), 0x21);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 6);

    // Absolute X
    cpu.pc = 0x2000;
    cpu.x = 0x01;
    bus.mem_write(0x2000, 0x5E); // LSR absolute,X
    bus.mem_write(0x2001, 0x33);
    bus.mem_write(0x2002, 0x12);
    bus.mem_write(0x1234, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(bus.mem_read(0x1234), 0x21);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 7);

    // Carry
    cpu.pc = 0x2000;
    cpu.a = 0x01;
    bus.mem_write(0x2000, 0x4A); // LSR A
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.a, 0x00);
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Zero
    cpu.pc = 0x2000;
    cpu.a = 0x00;
    bus.mem_write(0x2000, 0x4A); // LSR A
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.a, 0x00);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);
}

#[test]
fn test_rol() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Accumulator
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    bus.mem_write(0x2000, 0x2A); // ROL A
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.a, 0x84);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cycles, 2);

    // Zero page
    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0x26); // ROL $42
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x42, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(bus.mem_read(0x42), 0x84);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cycles, 5);

    // Zero page,X
    cpu.pc = 0x2000;
    cpu.x = 0x02;
    bus.mem_write(0x2000, 0x36); // ROL $40,X
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x42, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(bus.mem_read(0x42), 0x84);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cycles, 6);

    // Absolute
    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0x2E); // ROL $2042
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(bus.mem_read(0x2042), 0x84);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cycles, 6);

    // Absolute,X
    cpu.pc = 0x2000;
    cpu.x = 0x02;
    bus.mem_write(0x2000, 0x3E); // ROL $2040,X
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(bus.mem_read(0x2042), 0x84);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cycles, 7);
}

#[test]
fn test_ror() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Accumulator
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    bus.mem_write(0x2000, 0x6A); // ROR A
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.a, 0x21);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cycles, 2);

    // Zero page
    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0x66); // ROR $42
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x42, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(bus.mem_read(0x42), 0x21);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cycles, 5);

    // Zero page,X
    cpu.pc = 0x2000;
    cpu.x = 0x02;
    bus.mem_write(0x2000, 0x76); // ROR $40,X
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x42, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(bus.mem_read(0x42), 0x21);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cycles, 6);

    // Absolute
    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0x6E); // ROR $2042
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(bus.mem_read(0x2042), 0x21);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cycles, 6);

    // Absolute,X
    cpu.pc = 0x2000;
    cpu.x = 0x02;
    bus.mem_write(0x2000, 0x7E); // ROR $2040,X
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(bus.mem_read(0x2042), 0x21);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cycles, 7);
} 