use crate::cpu::w65c02::{Bus, CPU, bus::FakeBus, cpu::Flags};

#[test]
fn test_lda() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Immediate
    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0xA9); // LDA immediate
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x42);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Zero page
    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0xA5); // LDA zero page
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x42, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x42);
    assert_eq!(cycles, 3);

    // Zero page,X
    cpu.pc = 0x2000;
    cpu.x = 0x02;
    bus.mem_write(0x2000, 0xB5); // LDA zero page,X
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x42, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x42);
    assert_eq!(cycles, 4);

    // Absolute
    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0xAD); // LDA absolute
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cpu.a, 0x42);
    assert_eq!(cycles, 4);

    // Absolute,X
    cpu.pc = 0x2000;
    cpu.x = 0x02;
    bus.mem_write(0x2000, 0xBD); // LDA absolute,X
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cpu.a, 0x42);
    assert_eq!(cycles, 4);

    // Absolute,Y
    cpu.pc = 0x2000;
    cpu.y = 0x02;
    bus.mem_write(0x2000, 0xB9); // LDA absolute,Y
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cpu.a, 0x42);
    assert_eq!(cycles, 4);

    // (Indirect,X)
    cpu.pc = 0x2000;
    cpu.x = 0x02;
    bus.mem_write(0x2000, 0xA1); // LDA (indirect,X)
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x42, 0x42);
    bus.mem_write(0x43, 0x20);
    bus.mem_write(0x2042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x42);
    assert_eq!(cycles, 6);

    // (Indirect),Y
    cpu.pc = 0x2000;
    cpu.y = 0x02;
    bus.mem_write(0x2000, 0xB1); // LDA (indirect),Y
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x40, 0x40);
    bus.mem_write(0x41, 0x20);
    bus.mem_write(0x2042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x42);
    assert_eq!(cycles, 5);
}

#[test]
fn test_ldx() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Immediate
    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0xA2); // LDX immediate
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.x, 0x42);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Zero page
    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0xA6); // LDX zero page
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x42, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.x, 0x42);
    assert_eq!(cycles, 3);

    // Zero page,Y
    cpu.pc = 0x2000;
    cpu.y = 0x02;
    bus.mem_write(0x2000, 0xB6); // LDX zero page,Y
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x42, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.x, 0x42);
    assert_eq!(cycles, 4);

    // Absolute
    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0xAE); // LDX absolute
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cpu.x, 0x42);
    assert_eq!(cycles, 4);

    // Absolute,Y
    cpu.pc = 0x2000;
    cpu.y = 0x02;
    bus.mem_write(0x2000, 0xBE); // LDX absolute,Y
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cpu.x, 0x42);
    assert_eq!(cycles, 4);
}

#[test]
fn test_ldy() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Immediate
    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0xA0); // LDY immediate
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.y, 0x42);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Zero page
    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0xA4); // LDY zero page
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x42, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.y, 0x42);
    assert_eq!(cycles, 3);

    // Zero page,X
    cpu.pc = 0x2000;
    cpu.x = 0x02;
    bus.mem_write(0x2000, 0xB4); // LDY zero page,X
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x42, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.y, 0x42);
    assert_eq!(cycles, 4);

    // Absolute
    cpu.pc = 0x2000;
    bus.mem_write(0x2000, 0xAC); // LDY absolute
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cpu.y, 0x42);
    assert_eq!(cycles, 4);

    // Absolute,X
    cpu.pc = 0x2000;
    cpu.x = 0x02;
    bus.mem_write(0x2000, 0xBC); // LDY absolute,X
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cpu.y, 0x42);
    assert_eq!(cycles, 4);
}

#[test]
fn test_sta() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Zero page
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    bus.mem_write(0x2000, 0x85); // STA zero page
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(bus.mem_read(0x42), 0x42);
    assert_eq!(cycles, 3);

    // Zero page,X
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    cpu.x = 0x02;
    bus.mem_write(0x2000, 0x95); // STA zero page,X
    bus.mem_write(0x2001, 0x40);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(bus.mem_read(0x42), 0x42);
    assert_eq!(cycles, 4);

    // Absolute
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    bus.mem_write(0x2000, 0x8D); // STA absolute
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x2002, 0x20);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(bus.mem_read(0x2042), 0x42);
    assert_eq!(cycles, 4);

    // Absolute,X
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    cpu.x = 0x02;
    bus.mem_write(0x2000, 0x9D); // STA absolute,X
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x2002, 0x20);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(bus.mem_read(0x2042), 0x42);
    assert_eq!(cycles, 5);

    // Absolute,Y
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    cpu.y = 0x02;
    bus.mem_write(0x2000, 0x99); // STA absolute,Y
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x2002, 0x20);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(bus.mem_read(0x2042), 0x42);
    assert_eq!(cycles, 5);

    // (Indirect,X)
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    cpu.x = 0x02;
    bus.mem_write(0x2000, 0x81); // STA (indirect,X)
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x42, 0x42);
    bus.mem_write(0x43, 0x20);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(bus.mem_read(0x2042), 0x42);
    assert_eq!(cycles, 6);

    // (Indirect),Y
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    cpu.y = 0x02;
    bus.mem_write(0x2000, 0x91); // STA (indirect),Y
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x40, 0x40);
    bus.mem_write(0x41, 0x20);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(bus.mem_read(0x2042), 0x42);
    assert_eq!(cycles, 6);
}

#[test]
fn test_stx() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Zero page
    cpu.pc = 0x2000;
    cpu.x = 0x42;
    bus.mem_write(0x2000, 0x86); // STX zero page
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(bus.mem_read(0x42), 0x42);
    assert_eq!(cycles, 3);

    // Zero page,Y
    cpu.pc = 0x2000;
    cpu.x = 0x42;
    cpu.y = 0x02;
    bus.mem_write(0x2000, 0x96); // STX zero page,Y
    bus.mem_write(0x2001, 0x40);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(bus.mem_read(0x42), 0x42);
    assert_eq!(cycles, 4);

    // Absolute
    cpu.pc = 0x2000;
    cpu.x = 0x42;
    bus.mem_write(0x2000, 0x8E); // STX absolute
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x2002, 0x20);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(bus.mem_read(0x2042), 0x42);
    assert_eq!(cycles, 4);
}

#[test]
fn test_sty() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Zero page
    cpu.pc = 0x2000;
    cpu.y = 0x42;
    bus.mem_write(0x2000, 0x84); // STY zero page
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(bus.mem_read(0x42), 0x42);
    assert_eq!(cycles, 3);

    // Zero page,X
    cpu.pc = 0x2000;
    cpu.y = 0x42;
    cpu.x = 0x02;
    bus.mem_write(0x2000, 0x94); // STY zero page,X
    bus.mem_write(0x2001, 0x40);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(bus.mem_read(0x42), 0x42);
    assert_eq!(cycles, 4);

    // Absolute
    cpu.pc = 0x2000;
    cpu.y = 0x42;
    bus.mem_write(0x2000, 0x8C); // STY absolute
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x2002, 0x20);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(bus.mem_read(0x2042), 0x42);
    assert_eq!(cycles, 4);
}

#[test]
fn test_tax() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Positive value
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    bus.mem_write(0x2000, 0xAA); // TAX
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.x, 0x42);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Zero value
    cpu.pc = 0x2000;
    cpu.a = 0x00;
    bus.mem_write(0x2000, 0xAA); // TAX
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.x, 0x00);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Negative value
    cpu.pc = 0x2000;
    cpu.a = 0x80;
    bus.mem_write(0x2000, 0xAA); // TAX
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.x, 0x80);
    assert!(cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);
}

#[test]
fn test_tay() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Positive value
    cpu.pc = 0x2000;
    cpu.a = 0x42;
    bus.mem_write(0x2000, 0xA8); // TAY
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.y, 0x42);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Zero value
    cpu.pc = 0x2000;
    cpu.a = 0x00;
    bus.mem_write(0x2000, 0xA8); // TAY
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.y, 0x00);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Negative value
    cpu.pc = 0x2000;
    cpu.a = 0x80;
    bus.mem_write(0x2000, 0xA8); // TAY
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.y, 0x80);
    assert!(cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);
}

#[test]
fn test_tsx() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Positive value
    cpu.pc = 0x2000;
    cpu.sp = 0x42;
    bus.mem_write(0x2000, 0xBA); // TSX
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.x, 0x42);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Zero value
    cpu.pc = 0x2000;
    cpu.sp = 0x00;
    bus.mem_write(0x2000, 0xBA); // TSX
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.x, 0x00);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Negative value
    cpu.pc = 0x2000;
    cpu.sp = 0x80;
    bus.mem_write(0x2000, 0xBA); // TSX
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.x, 0x80);
    assert!(cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);
}

#[test]
fn test_txa() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Positive value
    cpu.pc = 0x2000;
    cpu.x = 0x42;
    bus.mem_write(0x2000, 0x8A); // TXA
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.a, 0x42);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Zero value
    cpu.pc = 0x2000;
    cpu.x = 0x00;
    bus.mem_write(0x2000, 0x8A); // TXA
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.a, 0x00);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Negative value
    cpu.pc = 0x2000;
    cpu.x = 0x80;
    bus.mem_write(0x2000, 0x8A); // TXA
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.a, 0x80);
    assert!(cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);
}

#[test]
fn test_txs() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Positive value
    cpu.pc = 0x2000;
    cpu.x = 0x42;
    bus.mem_write(0x2000, 0x9A); // TXS
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.sp, 0x42);
    assert_eq!(cycles, 2);

    // Zero value
    cpu.pc = 0x2000;
    cpu.x = 0x00;
    bus.mem_write(0x2000, 0x9A); // TXS
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.sp, 0x00);
    assert_eq!(cycles, 2);

    // Negative value
    cpu.pc = 0x2000;
    cpu.x = 0x80;
    bus.mem_write(0x2000, 0x9A); // TXS
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.sp, 0x80);
    assert_eq!(cycles, 2);
}

#[test]
fn test_tya() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Positive value
    cpu.pc = 0x2000;
    cpu.y = 0x42;
    bus.mem_write(0x2000, 0x98); // TYA
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.a, 0x42);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Zero value
    cpu.pc = 0x2000;
    cpu.y = 0x00;
    bus.mem_write(0x2000, 0x98); // TYA
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.a, 0x00);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);

    // Negative value
    cpu.pc = 0x2000;
    cpu.y = 0x80;
    bus.mem_write(0x2000, 0x98); // TYA
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.a, 0x80);
    assert!(cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert_eq!(cycles, 2);
} 