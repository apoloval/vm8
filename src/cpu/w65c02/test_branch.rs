use crate::cpu::w65c02::{Bus, CPU, bus::FakeBus, cpu::Flags};

#[test]
fn test_bcc() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Branch taken
    cpu.pc = 0x2000;
    cpu.status.remove(Flags::CARRY);
    bus.mem_write(0x2000, 0x90); // BCC
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2044);
    assert_eq!(cycles, 3);

    // Branch not taken
    cpu.pc = 0x2000;
    cpu.status.insert(Flags::CARRY);
    bus.mem_write(0x2000, 0x90); // BCC
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cycles, 2);

    // Branch taken, page cross
    cpu.pc = 0x20F0;
    cpu.status.remove(Flags::CARRY);
    bus.mem_write(0x20F0, 0x90); // BCC
    bus.mem_write(0x20F1, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2134);
    assert_eq!(cycles, 4);
}

#[test]
fn test_bcs() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Branch taken
    cpu.pc = 0x2000;
    cpu.status.insert(Flags::CARRY);
    bus.mem_write(0x2000, 0xB0); // BCS
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2044);
    assert_eq!(cycles, 3);

    // Branch not taken
    cpu.pc = 0x2000;
    cpu.status.remove(Flags::CARRY);
    bus.mem_write(0x2000, 0xB0); // BCS
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cycles, 2);

    // Branch taken, page cross
    cpu.pc = 0x20F0;
    cpu.status.insert(Flags::CARRY);
    bus.mem_write(0x20F0, 0xB0); // BCS
    bus.mem_write(0x20F1, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2134);
    assert_eq!(cycles, 4);
}

#[test]
fn test_beq() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Branch taken
    cpu.pc = 0x2000;
    cpu.status.insert(Flags::ZERO);
    bus.mem_write(0x2000, 0xF0); // BEQ
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2044);
    assert_eq!(cycles, 3);

    // Branch not taken
    cpu.pc = 0x2000;
    cpu.status.remove(Flags::ZERO);
    bus.mem_write(0x2000, 0xF0); // BEQ
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cycles, 2);

    // Branch taken, page cross
    cpu.pc = 0x20F0;
    cpu.status.insert(Flags::ZERO);
    bus.mem_write(0x20F0, 0xF0); // BEQ
    bus.mem_write(0x20F1, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2134);
    assert_eq!(cycles, 4);
}

#[test]
fn test_bmi() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Branch taken
    cpu.pc = 0x2000;
    cpu.status.insert(Flags::NEGATIVE);
    bus.mem_write(0x2000, 0x30); // BMI
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2044);
    assert_eq!(cycles, 3);

    // Branch not taken
    cpu.pc = 0x2000;
    cpu.status.remove(Flags::NEGATIVE);
    bus.mem_write(0x2000, 0x30); // BMI
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cycles, 2);

    // Branch taken, page cross
    cpu.pc = 0x20F0;
    cpu.status.insert(Flags::NEGATIVE);
    bus.mem_write(0x20F0, 0x30); // BMI
    bus.mem_write(0x20F1, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2134);
    assert_eq!(cycles, 4);
}

#[test]
fn test_bne() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Branch taken
    cpu.pc = 0x2000;
    cpu.status.remove(Flags::ZERO);
    bus.mem_write(0x2000, 0xD0); // BNE
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2044);
    assert_eq!(cycles, 3);

    // Branch not taken
    cpu.pc = 0x2000;
    cpu.status.insert(Flags::ZERO);
    bus.mem_write(0x2000, 0xD0); // BNE
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cycles, 2);

    // Branch taken, page cross
    cpu.pc = 0x20F0;
    cpu.status.remove(Flags::ZERO);
    bus.mem_write(0x20F0, 0xD0); // BNE
    bus.mem_write(0x20F1, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2134);
    assert_eq!(cycles, 4);
}

#[test]
fn test_bpl() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Branch taken
    cpu.pc = 0x2000;
    cpu.status.remove(Flags::NEGATIVE);
    bus.mem_write(0x2000, 0x10); // BPL
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2044);
    assert_eq!(cycles, 3);

    // Branch not taken
    cpu.pc = 0x2000;
    cpu.status.insert(Flags::NEGATIVE);
    bus.mem_write(0x2000, 0x10); // BPL
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cycles, 2);

    // Branch taken, page cross
    cpu.pc = 0x20F0;
    cpu.status.remove(Flags::NEGATIVE);
    bus.mem_write(0x20F0, 0x10); // BPL
    bus.mem_write(0x20F1, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2134);
    assert_eq!(cycles, 4);
}

#[test]
fn test_bvc() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Branch taken
    cpu.pc = 0x2000;
    cpu.status.remove(Flags::OVERFLOW);
    bus.mem_write(0x2000, 0x50); // BVC
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2044);
    assert_eq!(cycles, 3);

    // Branch not taken
    cpu.pc = 0x2000;
    cpu.status.insert(Flags::OVERFLOW);
    bus.mem_write(0x2000, 0x50); // BVC
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cycles, 2);

    // Branch taken, page cross
    cpu.pc = 0x20F0;
    cpu.status.remove(Flags::OVERFLOW);
    bus.mem_write(0x20F0, 0x50); // BVC
    bus.mem_write(0x20F1, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2134);
    assert_eq!(cycles, 4);
}

#[test]
fn test_bvs() {
    let mut cpu = CPU::new();
    let mut bus = FakeBus::new();

    // Branch taken
    cpu.pc = 0x2000;
    cpu.status.insert(Flags::OVERFLOW);
    bus.mem_write(0x2000, 0x70); // BVS
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2044);
    assert_eq!(cycles, 3);

    // Branch not taken
    cpu.pc = 0x2000;
    cpu.status.remove(Flags::OVERFLOW);
    bus.mem_write(0x2000, 0x70); // BVS
    bus.mem_write(0x2001, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2002);
    assert_eq!(cycles, 2);

    // Branch taken, page cross
    cpu.pc = 0x20F0;
    cpu.status.insert(Flags::OVERFLOW);
    bus.mem_write(0x20F0, 0x70); // BVS
    bus.mem_write(0x20F1, 0x42);
    let cycles = cpu.exec(&mut bus);
    assert_eq!(cpu.pc, 0x2134);
    assert_eq!(cycles, 4);
} 