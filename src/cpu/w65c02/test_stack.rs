use crate::cpu::w65c02::{Bus, CPU, bus::FakeBus, cpu::Flags, inst::Opcode};

#[test]
fn test_pha() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.a = 0x42;
    cpu.sp = 0xFF;
    bus.mem_write(0x2000, 0x48); // PHA
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.sp, 0xFE);
    assert_eq!(bus.mem_read(0x01FF), 0x42);
    assert!(matches!(inst.opcode, Opcode::PHA));
    assert_eq!(inst.cycles, 3);
}

#[test]
fn test_php() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.status = Flags::CARRY | Flags::ZERO;
    cpu.sp = 0xFF;
    bus.mem_write(0x2000, 0x08); // PHP
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.sp, 0xFE);
    let status = Flags::from_bits(bus.mem_read(0x01FF)).unwrap();
    assert_eq!(status, Flags::CARRY | Flags::ZERO | Flags::BREAK | Flags::UNUSED);
    assert!(matches!(inst.opcode, Opcode::PHP));
    assert_eq!(inst.cycles, 3);
}

#[test]
fn test_pla() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.sp = 0xFE;
    bus.mem_write(0x01FF, 0x42);
    bus.mem_write(0x2000, 0x68); // PLA
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.sp, 0xFF);
    assert_eq!(cpu.a, 0x42);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(matches!(inst.opcode, Opcode::PLA));
    assert_eq!(inst.cycles, 4);
}

#[test]
fn test_plp() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.sp = 0xFE;
    bus.mem_write(0x01FF, 0x03);
    bus.mem_write(0x2000, 0x28); // PLP
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.sp, 0xFF);
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(cpu.status.contains(Flags::ZERO));
    assert!(matches!(inst.opcode, Opcode::PLP));
    assert_eq!(inst.cycles, 4);
}

#[test]
fn test_tsx() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.sp = 0x42;
    bus.mem_write(0x2000, 0xBA); // TSX
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.x, 0x42);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(matches!(inst.opcode, Opcode::TSX));
    assert_eq!(inst.cycles, 2);
}

#[test]
fn test_txs() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.x = 0x42;
    bus.mem_write(0x2000, 0x9A); // TXS
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.sp, 0x42);
    assert!(matches!(inst.opcode, Opcode::TXS));
    assert_eq!(inst.cycles, 2);
} 