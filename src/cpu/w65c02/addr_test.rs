use crate::cpu::w65c02::{addr::{Mode, EffectiveAddress}, Bus, CPU, FakeBus};

#[test]
fn test_absolute() {
    let mut bus = FakeBus::new();
    let mut cpu = CPU::new();
    
    bus.mem_write(0x0000, 0x34);
    bus.mem_write(0x0001, 0x12);
    cpu.pc = 0x0000;
    
    let addr = Mode::Absolute.fetch(&mut cpu, &mut bus);
    assert!(matches!(addr, EffectiveAddress::Memory(0x1234)));
    assert_eq!(cpu.pc, 0x0002);
}

#[test]
fn test_absolute_x() {
    let mut bus = FakeBus::new();
    let mut cpu = CPU::new();
    
    bus.mem_write(0x0000, 0x34);
    bus.mem_write(0x0001, 0x12);
    cpu.pc = 0x0000;
    cpu.x = 0x01;
    
    let addr = Mode::AbsoluteX.fetch(&mut cpu, &mut bus);
    assert!(matches!(addr, EffectiveAddress::Memory(0x1235)));
    assert_eq!(cpu.pc, 0x0002);
}

#[test]
fn test_absolute_y() {
    let mut bus = FakeBus::new();
    let mut cpu = CPU::new();
    
    bus.mem_write(0x0000, 0x34);
    bus.mem_write(0x0001, 0x12);
    cpu.pc = 0x0000;
    cpu.y = 0x01;
    
    let addr = Mode::AbsoluteY.fetch(&mut cpu, &mut bus);
    assert!(matches!(addr, EffectiveAddress::Memory(0x1235)));
    assert_eq!(cpu.pc, 0x0002);
}

#[test]
fn test_accumulator() {
    let mut bus = FakeBus::new();
    let mut cpu = CPU::new();
    
    let addr = Mode::Accumulator.fetch(&mut cpu, &mut bus);
    assert!(matches!(addr, EffectiveAddress::Accumulator));
}

#[test]
fn test_immediate() {
    let mut bus = FakeBus::new();
    let mut cpu = CPU::new();
    
    cpu.pc = 0x1234;
    let addr = Mode::Immediate.fetch(&mut cpu, &mut bus);
    assert!(matches!(addr, EffectiveAddress::Memory(0x1234)));
    assert_eq!(cpu.pc, 0x1235);
}

#[test]
fn test_implied() {
    let mut bus = FakeBus::new();
    let mut cpu = CPU::new();
    
    let addr = Mode::Implied.fetch(&mut cpu, &mut bus);
    assert!(matches!(addr, EffectiveAddress::None));
}

#[test]
fn test_indirect() {
    let mut bus = FakeBus::new();
    let mut cpu = CPU::new();
    
    bus.mem_write(0x0000, 0x34);
    cpu.pc = 0x0000;
    bus.mem_write(0x0034, 0x78);
    bus.mem_write(0x0035, 0x56);
    
    let addr = Mode::Indirect.fetch(&mut cpu, &mut bus);
    assert!(matches!(addr, EffectiveAddress::Memory(0x5678)));
    assert_eq!(cpu.pc, 0x0001);
}

#[test]
fn test_indirect_x() {
    let mut bus = FakeBus::new();
    let mut cpu = CPU::new();
    
    bus.mem_write(0x0000, 0x34);
    cpu.pc = 0x0000;
    cpu.x = 0x01;
    bus.mem_write(0x0035, 0x78);
    bus.mem_write(0x0036, 0x56);
    
    let addr = Mode::IndirectX.fetch(&mut cpu, &mut bus);
    assert!(matches!(addr, EffectiveAddress::Memory(0x5678)));
    assert_eq!(cpu.pc, 0x0001);
}

#[test]
fn test_indirect_y() {
    let mut bus = FakeBus::new();
    let mut cpu = CPU::new();
    
    bus.mem_write(0x0000, 0x34);
    cpu.pc = 0x0000;
    cpu.y = 0x01;
    bus.mem_write(0x0034, 0x78);
    bus.mem_write(0x0035, 0x56);
    
    let addr = Mode::IndirectY.fetch(&mut cpu, &mut bus);
    assert!(matches!(addr, EffectiveAddress::Memory(0x5679)));
    assert_eq!(cpu.pc, 0x0001);
}

#[test]
fn test_relative() {
    let mut bus = FakeBus::new();
    let mut cpu = CPU::new();
    
    cpu.pc = 0x1234;
    let addr = Mode::Relative.fetch(&mut cpu, &mut bus);
    assert!(matches!(addr, EffectiveAddress::Memory(0x1234)));
    assert_eq!(cpu.pc, 0x1235);
}

#[test]
fn test_zero_page() {
    let mut bus = FakeBus::new();
    let mut cpu = CPU::new();
    
    bus.mem_write(0x0000, 0x34);
    cpu.pc = 0x0000;
    
    let addr = Mode::ZeroPage.fetch(&mut cpu, &mut bus);
    assert!(matches!(addr, EffectiveAddress::Memory(0x0034)));
    assert_eq!(cpu.pc, 0x0001);
}

#[test]
fn test_zero_page_x() {
    let mut bus = FakeBus::new();
    let mut cpu = CPU::new();
    
    bus.mem_write(0x0000, 0x34);
    cpu.pc = 0x0000;
    cpu.x = 0x01;
    
    let addr = Mode::ZeroPageX.fetch(&mut cpu, &mut bus);
    assert!(matches!(addr, EffectiveAddress::Memory(0x0035)));
    assert_eq!(cpu.pc, 0x0001);
}

#[test]
fn test_zero_page_y() {
    let mut bus = FakeBus::new();
    let mut cpu = CPU::new();
    
    bus.mem_write(0x0000, 0x34);
    cpu.pc = 0x0000;
    cpu.y = 0x01;
    
    let addr = Mode::ZeroPageY.fetch(&mut cpu, &mut bus);
    assert!(matches!(addr, EffectiveAddress::Memory(0x0035)));
    assert_eq!(cpu.pc, 0x0001);
}

#[test]
fn test_effective_address_read_write() {
    let mut bus = FakeBus::new();
    let mut cpu = CPU::new();
    
    cpu.a = 0x42;
    let acc_addr = EffectiveAddress::Accumulator;
    assert_eq!(acc_addr.read(&mut cpu, &mut bus), 0x42);
    
    let mem_addr = EffectiveAddress::Memory(0x1234);
    bus.mem_write(0x1234, 0x56);
    assert_eq!(mem_addr.read(&mut cpu, &mut bus), 0x56);
    
    mem_addr.write(&mut cpu, &mut bus, 0x78);
    assert_eq!(bus.mem_read(0x1234), 0x78);
    
    acc_addr.write(&mut cpu, &mut bus, 0x9A);
    assert_eq!(cpu.a, 0x9A);
    
    let none_addr = EffectiveAddress::None;
    assert_eq!(none_addr.read(&mut cpu, &mut bus), 0);
    none_addr.write(&mut cpu, &mut bus, 0xBC);
} 