use crate::cpu::w65c02::{Bus, CPU, bus::FakeBus, cpu::Flags, inst::Opcode};

#[test]
fn test_adc() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Immediate
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.status.set(Flags::CARRY, true);
    bus.mem_write(0x2000, 0x69); // ADC immediate
    bus.mem_write(0x2001, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x43);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::OVERFLOW));
    assert_eq!(inst.opcode, Opcode::ADC);
    assert_eq!(inst.cycles, 2);

    // Zero page
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.status.set(Flags::CARRY, true);
    bus.mem_write(0x2000, 0x65); // ADC zero page
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x42, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x43);
    assert_eq!(inst.opcode, Opcode::ADC);
    assert_eq!(inst.cycles, 3);

    // Zero page,X
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.x = 0x02;
    cpu.status.set(Flags::CARRY, true);
    bus.mem_write(0x2000, 0x75); // ADC zero page,X
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x42, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x43);
    assert_eq!(inst.opcode, Opcode::ADC);
    assert_eq!(inst.cycles, 4);

    // Absolute
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.status.set(Flags::CARRY, true);
    bus.mem_write(0x2000, 0x6D); // ADC absolute
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cpu.a, 0x43);
    assert_eq!(inst.opcode, Opcode::ADC);
    assert_eq!(inst.cycles, 4);

    // Absolute,X
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.x = 0x02;
    cpu.status.set(Flags::CARRY, true);
    bus.mem_write(0x2000, 0x7D); // ADC absolute,X
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cpu.a, 0x43);
    assert_eq!(inst.opcode, Opcode::ADC);
    assert_eq!(inst.cycles, 4);

    // Absolute,Y
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.y = 0x02;
    cpu.status.set(Flags::CARRY, true);
    bus.mem_write(0x2000, 0x79); // ADC absolute,Y
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cpu.a, 0x43);
    assert_eq!(inst.opcode, Opcode::ADC);
    assert_eq!(inst.cycles, 4);

    // (Indirect,X)
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.x = 0x02;
    cpu.status.set(Flags::CARRY, true);
    bus.mem_write(0x2000, 0x61); // ADC (indirect,X)
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x42, 0x42);
    bus.mem_write(0x43, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x43);
    assert_eq!(inst.opcode, Opcode::ADC);
    assert_eq!(inst.cycles, 6);

    // (Indirect),Y
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.y = 0x02;
    cpu.status.set(Flags::CARRY, true);
    bus.mem_write(0x2000, 0x71); // ADC (indirect),Y
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x40, 0x40);
    bus.mem_write(0x41, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x43);
    assert_eq!(inst.opcode, Opcode::ADC);
    assert_eq!(inst.cycles, 5);
}

#[test]
fn test_and() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Immediate
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    bus.mem_write(0x2000, 0x29); // AND immediate
    bus.mem_write(0x2001, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x21);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(matches!(inst.opcode, Opcode::AND));
    assert_eq!(inst.cycles, 2);

    // Zero
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    bus.mem_write(0x2000, 0x29); // AND immediate
    bus.mem_write(0x2001, 0x00);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x00);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert!(matches!(inst.opcode, Opcode::AND));
    assert_eq!(inst.cycles, 2);

    // Negative
    cpu.pc = 0x2000;
    cpu.a = 0x80;
    bus.mem_write(0x2000, 0x29); // AND immediate
    bus.mem_write(0x2001, 0x80);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x80);
    assert!(cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(matches!(inst.opcode, Opcode::AND));
    assert_eq!(inst.cycles, 2);
}

#[test]
fn test_eor() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Immediate
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    bus.mem_write(0x2000, 0x49); // EOR immediate
    bus.mem_write(0x2001, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x00);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert!(matches!(inst.opcode, Opcode::EOR));
    assert_eq!(inst.cycles, 2);

    // Zero page
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    bus.mem_write(0x2000, 0x45); // EOR zero page
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x42, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x00);
    assert_eq!(inst.opcode, Opcode::EOR);
    assert_eq!(inst.cycles, 3);

    // Zero page,X
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.x = 0x02;
    bus.mem_write(0x2000, 0x55); // EOR zero page,X
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x42, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x00);
    assert_eq!(inst.opcode, Opcode::EOR);
    assert_eq!(inst.cycles, 4);

    // Absolute
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    bus.mem_write(0x2000, 0x4D); // EOR absolute
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cpu.a, 0x00);
    assert_eq!(inst.opcode, Opcode::EOR);
    assert_eq!(inst.cycles, 4);

    // Absolute,X
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.x = 0x02;
    bus.mem_write(0x2000, 0x5D); // EOR absolute,X
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cpu.a, 0x00);
    assert_eq!(inst.opcode, Opcode::EOR);
    assert_eq!(inst.cycles, 4);

    // Absolute,Y
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.y = 0x02;
    bus.mem_write(0x2000, 0x59); // EOR absolute,Y
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cpu.a, 0x00);
    assert_eq!(inst.opcode, Opcode::EOR);
    assert_eq!(inst.cycles, 4);

    // (Indirect,X)
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.x = 0x02;
    bus.mem_write(0x2000, 0x41); // EOR (indirect,X)
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x42, 0x42);
    bus.mem_write(0x43, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x00);
    assert_eq!(inst.opcode, Opcode::EOR);
    assert_eq!(inst.cycles, 6);

    // (Indirect),Y
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.y = 0x02;
    bus.mem_write(0x2000, 0x51); // EOR (indirect),Y
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x40, 0x40);
    bus.mem_write(0x41, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x00);
    assert_eq!(inst.opcode, Opcode::EOR);
    assert_eq!(inst.cycles, 5);
}

#[test]
fn test_ora() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Immediate
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    bus.mem_write(0x2000, 0x09); // ORA immediate
    bus.mem_write(0x2001, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x21);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(matches!(inst.opcode, Opcode::ORA));
    assert_eq!(inst.cycles, 2);

    // Zero
    cpu.pc = 0x2000;
    cpu.a = 0x00;
    bus.mem_write(0x2000, 0x09); // ORA immediate
    bus.mem_write(0x2001, 0x00);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x00);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert!(matches!(inst.opcode, Opcode::ORA));
    assert_eq!(inst.cycles, 2);

    // Negative
    cpu.pc = 0x2000;
    cpu.a = 0x80;
    bus.mem_write(0x2000, 0x09); // ORA immediate
    bus.mem_write(0x2001, 0x80);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x80);
    assert!(cpu.status.contains(Flags::NEGATIVE));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(matches!(inst.opcode, Opcode::ORA));
    assert_eq!(inst.cycles, 2);
}

#[test]
fn test_cmp() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Equal
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    bus.mem_write(0x2000, 0xC9); // CMP immediate
    bus.mem_write(0x2001, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(matches!(inst.opcode, Opcode::CMP));
    assert_eq!(inst.cycles, 2);

    // Greater
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    bus.mem_write(0x2000, 0xC9); // CMP immediate
    bus.mem_write(0x2001, 0x20);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(matches!(inst.opcode, Opcode::CMP));
    assert_eq!(inst.cycles, 2);

    // Less
    cpu.pc = 0x2000;
    cpu.a = 0x20;
    bus.mem_write(0x2000, 0xC9); // CMP immediate
    bus.mem_write(0x2001, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert!(!cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(cpu.status.contains(Flags::NEGATIVE));
    assert!(matches!(inst.opcode, Opcode::CMP));
    assert_eq!(inst.cycles, 2);
}

#[test]
fn test_cpx() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Immediate
    cpu.pc = 0x2000;
    cpu.x = 0x21;
    bus.mem_write(0x2000, 0xE0); // CPX immediate
    bus.mem_write(0x2001, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(matches!(inst.opcode, Opcode::CPX));
    assert_eq!(inst.cycles, 2);

    // Zero page
    cpu.pc = 0x2000;
    cpu.x = 0x21;
    bus.mem_write(0x2000, 0xE4); // CPX zero page
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x42, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(matches!(inst.opcode, Opcode::CPX));
    assert_eq!(inst.cycles, 3);

    // Absolute
    cpu.pc = 0x2000;
    cpu.x = 0x21;
    bus.mem_write(0x2000, 0xEC); // CPX absolute
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(matches!(inst.opcode, Opcode::CPX));
    assert_eq!(inst.cycles, 4);
}

#[test]
fn test_cpy() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Immediate
    cpu.pc = 0x2000;
    cpu.y = 0x21;
    bus.mem_write(0x2000, 0xC0); // CPY immediate
    bus.mem_write(0x2001, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(matches!(inst.opcode, Opcode::CPY));
    assert_eq!(inst.cycles, 2);

    // Zero page
    cpu.pc = 0x2000;
    cpu.y = 0x21;
    bus.mem_write(0x2000, 0xC4); // CPY zero page
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x42, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(matches!(inst.opcode, Opcode::CPY));
    assert_eq!(inst.cycles, 3);

    // Absolute
    cpu.pc = 0x2000;
    cpu.y = 0x21;
    bus.mem_write(0x2000, 0xCC); // CPY absolute
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(matches!(inst.opcode, Opcode::CPY));
    assert_eq!(inst.cycles, 4);
}

#[test]
fn test_sbc() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Immediate
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.status.set(Flags::CARRY, true);
    bus.mem_write(0x2000, 0xE9); // SBC immediate
    bus.mem_write(0x2001, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x00);
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(cpu.status.contains(Flags::ZERO));
    assert!(cpu.status.contains(Flags::CARRY));
    assert!(!cpu.status.contains(Flags::OVERFLOW));
    assert_eq!(inst.opcode, Opcode::SBC);
    assert_eq!(inst.cycles, 2);

    // Zero page
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.status.set(Flags::CARRY, true);
    bus.mem_write(0x2000, 0xE5); // SBC zero page
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x42, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x00);
    assert_eq!(inst.opcode, Opcode::SBC);
    assert_eq!(inst.cycles, 3);

    // Zero page,X
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.x = 0x02;
    cpu.status.set(Flags::CARRY, true);
    bus.mem_write(0x2000, 0xF5); // SBC zero page,X
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x42, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x00);
    assert_eq!(inst.opcode, Opcode::SBC);
    assert_eq!(inst.cycles, 4);

    // Absolute
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.status.set(Flags::CARRY, true);
    bus.mem_write(0x2000, 0xED); // SBC absolute
    bus.mem_write(0x2001, 0x42);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cpu.a, 0x00);
    assert_eq!(inst.opcode, Opcode::SBC);
    assert_eq!(inst.cycles, 4);

    // Absolute,X
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.x = 0x02;
    cpu.status.set(Flags::CARRY, true);
    bus.mem_write(0x2000, 0xFD); // SBC absolute,X
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cpu.a, 0x00);
    assert_eq!(inst.opcode, Opcode::SBC);
    assert_eq!(inst.cycles, 4);

    // Absolute,Y
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.y = 0x02;
    cpu.status.set(Flags::CARRY, true);
    bus.mem_write(0x2000, 0xF9); // SBC absolute,Y
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x2002, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2003);
    assert_eq!(cpu.a, 0x00);
    assert_eq!(inst.opcode, Opcode::SBC);
    assert_eq!(inst.cycles, 4);

    // (Indirect,X)
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.x = 0x02;
    cpu.status.set(Flags::CARRY, true);
    bus.mem_write(0x2000, 0xE1); // SBC (indirect,X)
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x42, 0x42);
    bus.mem_write(0x43, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x00);
    assert_eq!(inst.opcode, Opcode::SBC);
    assert_eq!(inst.cycles, 6);

    // (Indirect),Y
    cpu.pc = 0x2000;
    cpu.a = 0x21;
    cpu.y = 0x02;
    cpu.status.set(Flags::CARRY, true);
    bus.mem_write(0x2000, 0xF1); // SBC (indirect),Y
    bus.mem_write(0x2001, 0x40);
    bus.mem_write(0x40, 0x40);
    bus.mem_write(0x41, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cpu.a, 0x00);
    assert_eq!(inst.opcode, Opcode::SBC);
    assert_eq!(inst.cycles, 5);
}

#[test]
fn test_inc() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Zero page
    cpu.pc = 0x0000;
    bus.mem_write(0x0000, 0xE6); // INC zero page
    bus.mem_write(0x0001, 0x42);
    bus.mem_write(0x0042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0002);
    assert_eq!(bus.mem_read(0x0042), 0x22);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(matches!(inst.opcode, Opcode::INC));
    assert_eq!(inst.cycles, 5);

    // Zero page X
    cpu.pc = 0x0000;
    cpu.x = 0x02;
    bus.mem_write(0x0000, 0xF6); // INC zero page,X
    bus.mem_write(0x0001, 0x40);
    bus.mem_write(0x0042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0002);
    assert_eq!(bus.mem_read(0x0042), 0x22);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(matches!(inst.opcode, Opcode::INC));
    assert_eq!(inst.cycles, 6);

    // Absolute
    cpu.pc = 0x0000;
    bus.mem_write(0x0000, 0xEE); // INC absolute
    bus.mem_write(0x0001, 0x42);
    bus.mem_write(0x0002, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0003);
    assert_eq!(bus.mem_read(0x0042), 0x22);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(matches!(inst.opcode, Opcode::INC));
    assert_eq!(inst.cycles, 6);

    // Absolute X
    cpu.pc = 0x0000;
    cpu.x = 0x02;
    bus.mem_write(0x0000, 0xFE); // INC absolute,X
    bus.mem_write(0x0001, 0x40);
    bus.mem_write(0x0002, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0003);
    assert_eq!(bus.mem_read(0x0042), 0x22);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(matches!(inst.opcode, Opcode::INC));
    assert_eq!(inst.cycles, 7);
}

#[test]
fn test_dec() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Zero page
    cpu.pc = 0x0000;
    bus.mem_write(0x0000, 0xC6); // DEC zero page
    bus.mem_write(0x0001, 0x42);
    bus.mem_write(0x0042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0002);
    assert_eq!(bus.mem_read(0x0042), 0x20);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(matches!(inst.opcode, Opcode::DEC));
    assert_eq!(inst.cycles, 5);

    // Zero page X
    cpu.pc = 0x0000;
    cpu.x = 0x02;
    bus.mem_write(0x0000, 0xD6); // DEC zero page,X
    bus.mem_write(0x0001, 0x40);
    bus.mem_write(0x0042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0002);
    assert_eq!(bus.mem_read(0x0042), 0x20);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(matches!(inst.opcode, Opcode::DEC));
    assert_eq!(inst.cycles, 6);

    // Absolute
    cpu.pc = 0x0000;
    bus.mem_write(0x0000, 0xCE); // DEC absolute
    bus.mem_write(0x0001, 0x42);
    bus.mem_write(0x0002, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0003);
    assert_eq!(bus.mem_read(0x0042), 0x20);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(matches!(inst.opcode, Opcode::DEC));
    assert_eq!(inst.cycles, 6);

    // Absolute X
    cpu.pc = 0x0000;
    cpu.x = 0x02;
    bus.mem_write(0x0000, 0xDE); // DEC absolute,X
    bus.mem_write(0x0001, 0x40);
    bus.mem_write(0x0002, 0x20);
    bus.mem_write(0x2042, 0x21);
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x0003);
    assert_eq!(bus.mem_read(0x0042), 0x20);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(matches!(inst.opcode, Opcode::DEC));
    assert_eq!(inst.cycles, 7);
}

#[test]
fn test_inx() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.x = 0x21;
    bus.mem_write(0x2000, 0xE8); // INX
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.x, 0x22);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(matches!(inst.opcode, Opcode::INX));
    assert_eq!(inst.cycles, 2);
}

#[test]
fn test_iny() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.y = 0x21;
    bus.mem_write(0x2000, 0xC8); // INY
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.y, 0x22);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(matches!(inst.opcode, Opcode::INY));
    assert_eq!(inst.cycles, 2);
}

#[test]
fn test_dex() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.x = 0x21;
    bus.mem_write(0x2000, 0xCA); // DEX
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.x, 0x20);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(matches!(inst.opcode, Opcode::DEX));
    assert_eq!(inst.cycles, 2);
}

#[test]
fn test_dey() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    cpu.pc = 0x2000;
    cpu.y = 0x21;
    bus.mem_write(0x2000, 0x88); // DEY
    let inst = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2001);
    assert_eq!(cpu.y, 0x20);
    assert!(!cpu.status.contains(Flags::ZERO));
    assert!(!cpu.status.contains(Flags::NEGATIVE));
    assert!(matches!(inst.opcode, Opcode::DEY));
    assert_eq!(inst.cycles, 2);
}
