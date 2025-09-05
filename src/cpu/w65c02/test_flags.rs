use crate::cpu::w65c02::{Bus, CPU, bus::FakeBus, cpu::Flags, inst::Opcode};

#[test]
fn test_clc() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.status.set(Flags::CARRY, true);
    bus.mem_write(0x2000, 0x18); // CLC
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(matches!(inst.opcode, Opcode::CLC));
    assert_eq!(inst.cycles, 2);
}

#[test]
fn test_cld() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.status.set(Flags::DECIMAL, true);
    bus.mem_write(0x2000, 0xD8); // CLD
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert!(!cpu.status.contains(Flags::DECIMAL));
    assert!(matches!(inst.opcode, Opcode::CLD));
    assert_eq!(inst.cycles, 2);
}

#[test]
fn test_cli() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.status.set(Flags::INTERRUPT, true);
    bus.mem_write(0x2000, 0x58); // CLI
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert!(!cpu.status.contains(Flags::INTERRUPT));
    assert!(matches!(inst.opcode, Opcode::CLI));
    assert_eq!(inst.cycles, 2);
}

#[test]
fn test_clv() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.status.set(Flags::OVERFLOW, true);
    bus.mem_write(0x2000, 0xB8); // CLV
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert!(!cpu.status.contains(Flags::OVERFLOW));
    assert!(matches!(inst.opcode, Opcode::CLV));
    assert_eq!(inst.cycles, 2);
}

#[test]
fn test_sec() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.status.set(Flags::CARRY, false);
    bus.mem_write(0x2000, 0x38); // SEC
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(matches!(inst.opcode, Opcode::SEC));
    assert_eq!(inst.cycles, 2);
}

#[test]
fn test_sed() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.status.set(Flags::DECIMAL, false);
    bus.mem_write(0x2000, 0xF8); // SED
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert!(cpu.status.contains(Flags::DECIMAL));
    assert!(matches!(inst.opcode, Opcode::SED));
    assert_eq!(inst.cycles, 2);
}

#[test]
fn test_sei() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.status.set(Flags::INTERRUPT, false);
    bus.mem_write(0x2000, 0x78); // SEI
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert!(cpu.status.contains(Flags::INTERRUPT));
    assert!(matches!(inst.opcode, Opcode::SEI));
    assert_eq!(inst.cycles, 2);
} 