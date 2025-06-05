use crate::cpu::w65c02::{Bus, CPU, bus::FakeBus, cpu::Flags};

#[test]
fn test_clc() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.status.insert(Flags::CARRY);
    bus.mem_write(0x2000, 0x18); // CLC
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert_eq!(cycles, 2);
}

#[test]
fn test_cld() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.status.insert(Flags::DECIMAL);
    bus.mem_write(0x2000, 0xD8); // CLD
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert!(!cpu.status.contains(Flags::DECIMAL));
    assert_eq!(cycles, 2);
}

#[test]
fn test_cli() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.status.insert(Flags::INTERRUPT);
    bus.mem_write(0x2000, 0x58); // CLI
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert!(!cpu.status.contains(Flags::INTERRUPT));
    assert_eq!(cycles, 2);
}

#[test]
fn test_clv() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.status.insert(Flags::OVERFLOW);
    bus.mem_write(0x2000, 0xB8); // CLV
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert!(!cpu.status.contains(Flags::OVERFLOW));
    assert_eq!(cycles, 2);
}

#[test]
fn test_sec() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.status.remove(Flags::CARRY);
    bus.mem_write(0x2000, 0x38); // SEC
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert!(cpu.status.contains(Flags::CARRY));
    assert_eq!(cycles, 2);
}

#[test]
fn test_sed() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.status.remove(Flags::DECIMAL);
    bus.mem_write(0x2000, 0xF8); // SED
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert!(cpu.status.contains(Flags::DECIMAL));
    assert_eq!(cycles, 2);
}

#[test]
fn test_sei() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.status.remove(Flags::INTERRUPT);
    bus.mem_write(0x2000, 0x78); // SEI
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert!(cpu.status.contains(Flags::INTERRUPT));
    assert_eq!(cycles, 2);
} 