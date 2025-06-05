use crate::cpu::w65c02::{Bus, CPU, bus::FakeBus, cpu::Flags};

#[test]
fn test_pha() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.sp = 0xFF;
    cpu.a = 0x42;
    bus.mem_write(0x2000, 0x48); // PHA
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.sp, 0xFE);
    assert_eq!(bus.mem_read(0x01FF), 0x42);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cycles, 3);
}

#[test]
fn test_php() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.sp = 0xFF;
    cpu.status = Flags::CARRY | Flags::NEGATIVE;
    bus.mem_write(0x2000, 0x08); // PHP
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.sp, 0xFE);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cycles, 3);

    let status = Flags::from_bits(bus.mem_read(0x01FF)).unwrap();
    assert!(status.contains(Flags::CARRY));
    assert!(status.contains(Flags::NEGATIVE));
    assert!(status.contains(Flags::BREAK));
    assert!(status.contains(Flags::UNUSED));
}

#[test]
fn test_pla() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.sp = 0xFE;
    bus.mem_write(0x01FF, 0x42);
    bus.mem_write(0x2000, 0x68); // PLA
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.sp, 0xFF);
    assert_eq!(cpu.a, 0x42);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cycles, 4);
}

#[test]
fn test_plp() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.sp = 0xFE;
    bus.mem_write(0x01FF, 0x81);
    bus.mem_write(0x2000, 0x28); // PLP
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.sp, 0xFF);
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(cpu.status.contains(Flags::NEGATIVE));
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cycles, 4);
}

#[test]
fn test_tsx() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.sp = 0x42;
    bus.mem_write(0x2000, 0xBA); // TSX
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.x, 0x42);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cycles, 2);
}

#[test]
fn test_txs() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.x = 0x42;
    bus.mem_write(0x2000, 0x9A); // TXS
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.sp, 0x42);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cycles, 2);
} 