use crate::cpu::w65c02::{addr::Mode, addr::EffectiveAddress, Bus, CPU, cpu::Flags};

pub enum Opcode {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
}

/// An instruction handler takes a CPU, a bus, and an instruction, and returns the number of cycles 
/// taken to execute the instruction.
pub type Handler<B> = fn(&mut CPU, &mut B, &Instruction<B>) -> usize;


pub struct Instruction<B: Bus> {
    pub opcode: Opcode,
    pub cycles: usize,
    pub address_mode: Mode,
    pub handler: Handler<B>,
}

pub fn decode<B: Bus>(cpu: &mut CPU, bus: &mut B) -> Instruction<B> {
    match cpu.fetch_byte(bus) {
        // ADC
        0x69 => Instruction { opcode: Opcode::ADC, cycles: 2, address_mode: Mode::Immediate, handler: handle_adc },
        0x65 => Instruction { opcode: Opcode::ADC, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_adc },
        0x75 => Instruction { opcode: Opcode::ADC, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_adc },
        0x6D => Instruction { opcode: Opcode::ADC, cycles: 4, address_mode: Mode::Absolute, handler: handle_adc },
        0x7D => Instruction { opcode: Opcode::ADC, cycles: 4, address_mode: Mode::AbsoluteX, handler: handle_adc },
        0x79 => Instruction { opcode: Opcode::ADC, cycles: 4, address_mode: Mode::AbsoluteY, handler: handle_adc },
        0x61 => Instruction { opcode: Opcode::ADC, cycles: 6, address_mode: Mode::IndirectX, handler: handle_adc },
        0x71 => Instruction { opcode: Opcode::ADC, cycles: 5, address_mode: Mode::IndirectY, handler: handle_adc },

        // AND
        0x29 => Instruction { opcode: Opcode::AND, cycles: 2, address_mode: Mode::Immediate, handler: handle_and },
        0x25 => Instruction { opcode: Opcode::AND, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_and },
        0x35 => Instruction { opcode: Opcode::AND, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_and },
        0x2D => Instruction { opcode: Opcode::AND, cycles: 4, address_mode: Mode::Absolute, handler: handle_and },
        0x3D => Instruction { opcode: Opcode::AND, cycles: 4, address_mode: Mode::AbsoluteX, handler: handle_and },
        0x39 => Instruction { opcode: Opcode::AND, cycles: 4, address_mode: Mode::AbsoluteY, handler: handle_and },
        0x21 => Instruction { opcode: Opcode::AND, cycles: 6, address_mode: Mode::IndirectX, handler: handle_and },
        0x31 => Instruction { opcode: Opcode::AND, cycles: 5, address_mode: Mode::IndirectY, handler: handle_and },

        // ASL
        0x0A => Instruction { opcode: Opcode::ASL, cycles: 2, address_mode: Mode::Accumulator, handler: handle_asl },
        0x06 => Instruction { opcode: Opcode::ASL, cycles: 5, address_mode: Mode::ZeroPage, handler: handle_asl },
        0x16 => Instruction { opcode: Opcode::ASL, cycles: 6, address_mode: Mode::ZeroPageX, handler: handle_asl },
        0x0E => Instruction { opcode: Opcode::ASL, cycles: 6, address_mode: Mode::Absolute, handler: handle_asl },
        0x1E => Instruction { opcode: Opcode::ASL, cycles: 7, address_mode: Mode::AbsoluteX, handler: handle_asl },

        // Branch instructions
        0x90 => Instruction { opcode: Opcode::BCC, cycles: 2, address_mode: Mode::Relative, handler: handle_bcc },
        0xB0 => Instruction { opcode: Opcode::BCS, cycles: 2, address_mode: Mode::Relative, handler: handle_bcs },
        0xF0 => Instruction { opcode: Opcode::BEQ, cycles: 2, address_mode: Mode::Relative, handler: handle_beq },
        0x30 => Instruction { opcode: Opcode::BMI, cycles: 2, address_mode: Mode::Relative, handler: handle_bmi },
        0xD0 => Instruction { opcode: Opcode::BNE, cycles: 2, address_mode: Mode::Relative, handler: handle_bne },
        0x10 => Instruction { opcode: Opcode::BPL, cycles: 2, address_mode: Mode::Relative, handler: handle_bpl },
        0x50 => Instruction { opcode: Opcode::BVC, cycles: 2, address_mode: Mode::Relative, handler: handle_bvc },
        0x70 => Instruction { opcode: Opcode::BVS, cycles: 2, address_mode: Mode::Relative, handler: handle_bvs },

        // BIT
        0x24 => Instruction { opcode: Opcode::BIT, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_bit },
        0x2C => Instruction { opcode: Opcode::BIT, cycles: 4, address_mode: Mode::Absolute, handler: handle_bit },

        // BRK
        0x00 => Instruction { opcode: Opcode::BRK, cycles: 7, address_mode: Mode::Implied, handler: handle_brk },

        // Clear flags
        0x18 => Instruction { opcode: Opcode::CLC, cycles: 2, address_mode: Mode::Implied, handler: handle_clc },
        0xD8 => Instruction { opcode: Opcode::CLD, cycles: 2, address_mode: Mode::Implied, handler: handle_cld },
        0x58 => Instruction { opcode: Opcode::CLI, cycles: 2, address_mode: Mode::Implied, handler: handle_cli },
        0xB8 => Instruction { opcode: Opcode::CLV, cycles: 2, address_mode: Mode::Implied, handler: handle_clv },

        // Compare
        0xC9 => Instruction { opcode: Opcode::CMP, cycles: 2, address_mode: Mode::Immediate, handler: handle_cmp },
        0xC5 => Instruction { opcode: Opcode::CMP, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_cmp },
        0xD5 => Instruction { opcode: Opcode::CMP, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_cmp },
        0xCD => Instruction { opcode: Opcode::CMP, cycles: 4, address_mode: Mode::Absolute, handler: handle_cmp },
        0xDD => Instruction { opcode: Opcode::CMP, cycles: 4, address_mode: Mode::AbsoluteX, handler: handle_cmp },
        0xD9 => Instruction { opcode: Opcode::CMP, cycles: 4, address_mode: Mode::AbsoluteY, handler: handle_cmp },
        0xC1 => Instruction { opcode: Opcode::CMP, cycles: 6, address_mode: Mode::IndirectX, handler: handle_cmp },
        0xD1 => Instruction { opcode: Opcode::CMP, cycles: 5, address_mode: Mode::IndirectY, handler: handle_cmp },

        // CPX
        0xE0 => Instruction { opcode: Opcode::CPX, cycles: 2, address_mode: Mode::Immediate, handler: handle_cpx },
        0xE4 => Instruction { opcode: Opcode::CPX, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_cpx },
        0xEC => Instruction { opcode: Opcode::CPX, cycles: 4, address_mode: Mode::Absolute, handler: handle_cpx },

        // CPY
        0xC0 => Instruction { opcode: Opcode::CPY, cycles: 2, address_mode: Mode::Immediate, handler: handle_cpy },
        0xC4 => Instruction { opcode: Opcode::CPY, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_cpy },
        0xCC => Instruction { opcode: Opcode::CPY, cycles: 4, address_mode: Mode::Absolute, handler: handle_cpy },

        // DEC
        0xC6 => Instruction { opcode: Opcode::DEC, cycles: 5, address_mode: Mode::ZeroPage, handler: handle_dec },
        0xD6 => Instruction { opcode: Opcode::DEC, cycles: 6, address_mode: Mode::ZeroPageX, handler: handle_dec },
        0xCE => Instruction { opcode: Opcode::DEC, cycles: 6, address_mode: Mode::Absolute, handler: handle_dec },
        0xDE => Instruction { opcode: Opcode::DEC, cycles: 7, address_mode: Mode::AbsoluteX, handler: handle_dec },

        // Decrement registers
        0xCA => Instruction { opcode: Opcode::DEX, cycles: 2, address_mode: Mode::Implied, handler: handle_dex },
        0x88 => Instruction { opcode: Opcode::DEY, cycles: 2, address_mode: Mode::Implied, handler: handle_dey },

        // EOR
        0x49 => Instruction { opcode: Opcode::EOR, cycles: 2, address_mode: Mode::Immediate, handler: handle_eor },
        0x45 => Instruction { opcode: Opcode::EOR, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_eor },
        0x55 => Instruction { opcode: Opcode::EOR, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_eor },
        0x4D => Instruction { opcode: Opcode::EOR, cycles: 4, address_mode: Mode::Absolute, handler: handle_eor },
        0x5D => Instruction { opcode: Opcode::EOR, cycles: 4, address_mode: Mode::AbsoluteX, handler: handle_eor },
        0x59 => Instruction { opcode: Opcode::EOR, cycles: 4, address_mode: Mode::AbsoluteY, handler: handle_eor },
        0x41 => Instruction { opcode: Opcode::EOR, cycles: 6, address_mode: Mode::IndirectX, handler: handle_eor },
        0x51 => Instruction { opcode: Opcode::EOR, cycles: 5, address_mode: Mode::IndirectY, handler: handle_eor },

        // INC
        0xE6 => Instruction { opcode: Opcode::INC, cycles: 5, address_mode: Mode::ZeroPage, handler: handle_inc },
        0xF6 => Instruction { opcode: Opcode::INC, cycles: 6, address_mode: Mode::ZeroPageX, handler: handle_inc },
        0xEE => Instruction { opcode: Opcode::INC, cycles: 6, address_mode: Mode::Absolute, handler: handle_inc },
        0xFE => Instruction { opcode: Opcode::INC, cycles: 7, address_mode: Mode::AbsoluteX, handler: handle_inc },

        // Increment registers
        0xE8 => Instruction { opcode: Opcode::INX, cycles: 2, address_mode: Mode::Implied, handler: handle_inx },
        0xC8 => Instruction { opcode: Opcode::INY, cycles: 2, address_mode: Mode::Implied, handler: handle_iny },

        // JMP
        0x4C => Instruction { opcode: Opcode::JMP, cycles: 3, address_mode: Mode::Absolute, handler: handle_jmp },
        0x6C => Instruction { opcode: Opcode::JMP, cycles: 5, address_mode: Mode::Indirect, handler: handle_jmp },

        // JSR
        0x20 => Instruction { opcode: Opcode::JSR, cycles: 6, address_mode: Mode::Absolute, handler: handle_jsr },

        // LDA
        0xA9 => Instruction { opcode: Opcode::LDA, cycles: 2, address_mode: Mode::Immediate, handler: handle_lda },
        0xA5 => Instruction { opcode: Opcode::LDA, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_lda },
        0xB5 => Instruction { opcode: Opcode::LDA, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_lda },
        0xAD => Instruction { opcode: Opcode::LDA, cycles: 4, address_mode: Mode::Absolute, handler: handle_lda },
        0xBD => Instruction { opcode: Opcode::LDA, cycles: 4, address_mode: Mode::AbsoluteX, handler: handle_lda },
        0xB9 => Instruction { opcode: Opcode::LDA, cycles: 4, address_mode: Mode::AbsoluteY, handler: handle_lda },
        0xA1 => Instruction { opcode: Opcode::LDA, cycles: 6, address_mode: Mode::IndirectX, handler: handle_lda },
        0xB1 => Instruction { opcode: Opcode::LDA, cycles: 5, address_mode: Mode::IndirectY, handler: handle_lda },

        // LDX
        0xA2 => Instruction { opcode: Opcode::LDX, cycles: 2, address_mode: Mode::Immediate, handler: handle_ldx },
        0xA6 => Instruction { opcode: Opcode::LDX, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_ldx },
        0xB6 => Instruction { opcode: Opcode::LDX, cycles: 4, address_mode: Mode::ZeroPageY, handler: handle_ldx },
        0xAE => Instruction { opcode: Opcode::LDX, cycles: 4, address_mode: Mode::Absolute, handler: handle_ldx },
        0xBE => Instruction { opcode: Opcode::LDX, cycles: 4, address_mode: Mode::AbsoluteY, handler: handle_ldx },

        // LDY
        0xA0 => Instruction { opcode: Opcode::LDY, cycles: 2, address_mode: Mode::Immediate, handler: handle_ldy },
        0xA4 => Instruction { opcode: Opcode::LDY, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_ldy },
        0xB4 => Instruction { opcode: Opcode::LDY, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_ldy },
        0xAC => Instruction { opcode: Opcode::LDY, cycles: 4, address_mode: Mode::Absolute, handler: handle_ldy },
        0xBC => Instruction { opcode: Opcode::LDY, cycles: 4, address_mode: Mode::AbsoluteX, handler: handle_ldy },

        // LSR
        0x4A => Instruction { opcode: Opcode::LSR, cycles: 2, address_mode: Mode::Accumulator, handler: handle_lsr },
        0x46 => Instruction { opcode: Opcode::LSR, cycles: 5, address_mode: Mode::ZeroPage, handler: handle_lsr },
        0x56 => Instruction { opcode: Opcode::LSR, cycles: 6, address_mode: Mode::ZeroPageX, handler: handle_lsr },
        0x4E => Instruction { opcode: Opcode::LSR, cycles: 6, address_mode: Mode::Absolute, handler: handle_lsr },
        0x5E => Instruction { opcode: Opcode::LSR, cycles: 7, address_mode: Mode::AbsoluteX, handler: handle_lsr },

        // NOP
        0xEA => Instruction { opcode: Opcode::NOP, cycles: 2, address_mode: Mode::Implied, handler: handle_nop },

        // ORA
        0x09 => Instruction { opcode: Opcode::ORA, cycles: 2, address_mode: Mode::Immediate, handler: handle_ora },
        0x05 => Instruction { opcode: Opcode::ORA, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_ora },
        0x15 => Instruction { opcode: Opcode::ORA, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_ora },
        0x0D => Instruction { opcode: Opcode::ORA, cycles: 4, address_mode: Mode::Absolute, handler: handle_ora },
        0x1D => Instruction { opcode: Opcode::ORA, cycles: 4, address_mode: Mode::AbsoluteX, handler: handle_ora },
        0x19 => Instruction { opcode: Opcode::ORA, cycles: 4, address_mode: Mode::AbsoluteY, handler: handle_ora },
        0x01 => Instruction { opcode: Opcode::ORA, cycles: 6, address_mode: Mode::IndirectX, handler: handle_ora },
        0x11 => Instruction { opcode: Opcode::ORA, cycles: 5, address_mode: Mode::IndirectY, handler: handle_ora },

        // Stack operations
        0x48 => Instruction { opcode: Opcode::PHA, cycles: 3, address_mode: Mode::Implied, handler: handle_pha },
        0x08 => Instruction { opcode: Opcode::PHP, cycles: 3, address_mode: Mode::Implied, handler: handle_php },
        0x68 => Instruction { opcode: Opcode::PLA, cycles: 4, address_mode: Mode::Implied, handler: handle_pla },
        0x28 => Instruction { opcode: Opcode::PLP, cycles: 4, address_mode: Mode::Implied, handler: handle_plp },

        // ROL
        0x2A => Instruction { opcode: Opcode::ROL, cycles: 2, address_mode: Mode::Accumulator, handler: handle_rol },
        0x26 => Instruction { opcode: Opcode::ROL, cycles: 5, address_mode: Mode::ZeroPage, handler: handle_rol },
        0x36 => Instruction { opcode: Opcode::ROL, cycles: 6, address_mode: Mode::ZeroPageX, handler: handle_rol },
        0x2E => Instruction { opcode: Opcode::ROL, cycles: 6, address_mode: Mode::Absolute, handler: handle_rol },
        0x3E => Instruction { opcode: Opcode::ROL, cycles: 7, address_mode: Mode::AbsoluteX, handler: handle_rol },

        // ROR
        0x6A => Instruction { opcode: Opcode::ROR, cycles: 2, address_mode: Mode::Accumulator, handler: handle_ror },
        0x66 => Instruction { opcode: Opcode::ROR, cycles: 5, address_mode: Mode::ZeroPage, handler: handle_ror },
        0x76 => Instruction { opcode: Opcode::ROR, cycles: 6, address_mode: Mode::ZeroPageX, handler: handle_ror },
        0x6E => Instruction { opcode: Opcode::ROR, cycles: 6, address_mode: Mode::Absolute, handler: handle_ror },
        0x7E => Instruction { opcode: Opcode::ROR, cycles: 7, address_mode: Mode::AbsoluteX, handler: handle_ror },

        // RTI
        0x40 => Instruction { opcode: Opcode::RTI, cycles: 6, address_mode: Mode::Implied, handler: handle_rti },

        // RTS
        0x60 => Instruction { opcode: Opcode::RTS, cycles: 6, address_mode: Mode::Implied, handler: handle_rts },

        // SBC
        0xE9 => Instruction { opcode: Opcode::SBC, cycles: 2, address_mode: Mode::Immediate, handler: handle_sbc },
        0xE5 => Instruction { opcode: Opcode::SBC, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_sbc },
        0xF5 => Instruction { opcode: Opcode::SBC, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_sbc },
        0xED => Instruction { opcode: Opcode::SBC, cycles: 4, address_mode: Mode::Absolute, handler: handle_sbc },
        0xFD => Instruction { opcode: Opcode::SBC, cycles: 4, address_mode: Mode::AbsoluteX, handler: handle_sbc },
        0xF9 => Instruction { opcode: Opcode::SBC, cycles: 4, address_mode: Mode::AbsoluteY, handler: handle_sbc },
        0xE1 => Instruction { opcode: Opcode::SBC, cycles: 6, address_mode: Mode::IndirectX, handler: handle_sbc },
        0xF1 => Instruction { opcode: Opcode::SBC, cycles: 5, address_mode: Mode::IndirectY, handler: handle_sbc },

        // Set flags
        0x38 => Instruction { opcode: Opcode::SEC, cycles: 2, address_mode: Mode::Implied, handler: handle_sec },
        0xF8 => Instruction { opcode: Opcode::SED, cycles: 2, address_mode: Mode::Implied, handler: handle_sed },
        0x78 => Instruction { opcode: Opcode::SEI, cycles: 2, address_mode: Mode::Implied, handler: handle_sei },

        // Store instructions
        0x85 => Instruction { opcode: Opcode::STA, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_sta },
        0x95 => Instruction { opcode: Opcode::STA, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_sta },
        0x8D => Instruction { opcode: Opcode::STA, cycles: 4, address_mode: Mode::Absolute, handler: handle_sta },
        0x9D => Instruction { opcode: Opcode::STA, cycles: 5, address_mode: Mode::AbsoluteX, handler: handle_sta },
        0x99 => Instruction { opcode: Opcode::STA, cycles: 5, address_mode: Mode::AbsoluteY, handler: handle_sta },
        0x81 => Instruction { opcode: Opcode::STA, cycles: 6, address_mode: Mode::IndirectX, handler: handle_sta },
        0x91 => Instruction { opcode: Opcode::STA, cycles: 6, address_mode: Mode::IndirectY, handler: handle_sta },

        0x86 => Instruction { opcode: Opcode::STX, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_stx },
        0x96 => Instruction { opcode: Opcode::STX, cycles: 4, address_mode: Mode::ZeroPageY, handler: handle_stx },
        0x8E => Instruction { opcode: Opcode::STX, cycles: 4, address_mode: Mode::Absolute, handler: handle_stx },

        0x84 => Instruction { opcode: Opcode::STY, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_sty },
        0x94 => Instruction { opcode: Opcode::STY, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_sty },
        0x8C => Instruction { opcode: Opcode::STY, cycles: 4, address_mode: Mode::Absolute, handler: handle_sty },

        // Transfer instructions
        0xAA => Instruction { opcode: Opcode::TAX, cycles: 2, address_mode: Mode::Implied, handler: handle_tax },
        0xA8 => Instruction { opcode: Opcode::TAY, cycles: 2, address_mode: Mode::Implied, handler: handle_tay },
        0xBA => Instruction { opcode: Opcode::TSX, cycles: 2, address_mode: Mode::Implied, handler: handle_tsx },
        0x8A => Instruction { opcode: Opcode::TXA, cycles: 2, address_mode: Mode::Implied, handler: handle_txa },
        0x9A => Instruction { opcode: Opcode::TXS, cycles: 2, address_mode: Mode::Implied, handler: handle_txs },
        0x98 => Instruction { opcode: Opcode::TYA, cycles: 2, address_mode: Mode::Implied, handler: handle_tya },

        _ => panic!("Invalid opcode: {:02X}", cpu.pc - 1),
    }
}

// Handler stubs for each opcode
fn handle_adc<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    let value = addr.read(cpu, bus);
    let carry = if cpu.status.contains(Flags::CARRY) { 1 } else { 0 };
    
    // First do binary addition
    let binary_result = cpu.a.wrapping_add(value).wrapping_add(carry);
    
    // Calculate flags based on binary result
    Flags::set(&mut cpu.status, Flags::CARRY, binary_result < cpu.a || (binary_result == cpu.a && carry == 1));
    Flags::set(&mut cpu.status, Flags::OVERFLOW, (cpu.a ^ binary_result) & (value ^ binary_result) & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::NEGATIVE, binary_result & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, binary_result == 0);

    // If in BCD mode, adjust the result
    let result = if cpu.status.contains(Flags::DECIMAL) {
        let mut al = binary_result & 0x0F;
        let mut ah = binary_result >> 4;
        
        if al > 9 {
            al += 6;
            ah += 1;
        }
        
        if ah > 9 {
            ah += 6;
            cpu.status.insert(Flags::CARRY);
        }
        
        ((ah & 0x0F) << 4) | (al & 0x0F)
    } else {
        binary_result
    };

    cpu.a = result;

    inst.cycles
}

fn handle_and<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    let value = addr.read(cpu, bus);
    cpu.a &= value;

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.a & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.a == 0);

    inst.cycles
}

fn handle_asl<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    let value = addr.read(cpu, bus);
    let result = value << 1;

    Flags::set(&mut cpu.status, Flags::CARRY, value & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::NEGATIVE, result & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    match inst.address_mode {
        Mode::Accumulator => cpu.a = result,
        _ => addr.write(cpu, bus, result),
    }

    inst.cycles
}

fn handle_bcc<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    if !cpu.status.contains(Flags::CARRY) {
        let offset = addr.read(cpu, bus) as i8;
        let old_pc = cpu.pc;
        cpu.pc = cpu.pc.wrapping_add(offset as u16);
        inst.cycles + if (old_pc & 0xFF00) != (cpu.pc & 0xFF00) { 2 } else { 1 }
    } else {
        inst.cycles
    }
}

fn handle_bcs<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    if cpu.status.contains(Flags::CARRY) {
        let offset = addr.read(cpu, bus) as i8;
        let old_pc = cpu.pc;
        cpu.pc = cpu.pc.wrapping_add(offset as u16);
        inst.cycles + if (old_pc & 0xFF00) != (cpu.pc & 0xFF00) { 2 } else { 1 }
    } else {
        inst.cycles
    }
}

fn handle_beq<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    if cpu.status.contains(Flags::ZERO) {
        let offset = addr.read(cpu, bus) as i8;
        let old_pc = cpu.pc;
        cpu.pc = cpu.pc.wrapping_add(offset as u16);
        inst.cycles + if (old_pc & 0xFF00) != (cpu.pc & 0xFF00) { 2 } else { 1 }
    } else {
        inst.cycles
    }
}

fn handle_bit<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    let value = addr.read(cpu, bus);
    let result = cpu.a & value;

    Flags::set(&mut cpu.status, Flags::NEGATIVE, value & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::OVERFLOW, value & 0x40 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    inst.cycles
}

fn handle_bmi<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    if cpu.status.contains(Flags::NEGATIVE) {
        let offset = addr.read(cpu, bus) as i8;
        let old_pc = cpu.pc;
        cpu.pc = cpu.pc.wrapping_add(offset as u16);
        inst.cycles + if (old_pc & 0xFF00) != (cpu.pc & 0xFF00) { 2 } else { 1 }
    } else {
        inst.cycles
    }
}

fn handle_bne<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    if !cpu.status.contains(Flags::ZERO) {
        let offset = addr.read(cpu, bus) as i8;
        let old_pc = cpu.pc;
        cpu.pc = cpu.pc.wrapping_add(offset as u16);
        inst.cycles + if (old_pc & 0xFF00) != (cpu.pc & 0xFF00) { 2 } else { 1 }
    } else {
        inst.cycles
    }
}

fn handle_bpl<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    if !cpu.status.contains(Flags::NEGATIVE) {
        let offset = addr.read(cpu, bus) as i8;
        let old_pc = cpu.pc;
        cpu.pc = cpu.pc.wrapping_add(offset as u16);
        inst.cycles + if (old_pc & 0xFF00) != (cpu.pc & 0xFF00) { 2 } else { 1 }
    } else {
        inst.cycles
    }
}

fn handle_brk<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    // Set the right flags in the right statuses
    let mut status_copy = cpu.status;
    status_copy.insert(Flags::BREAK);
    cpu.status.insert(Flags::INTERRUPT);
    
    // Push PC+2 and status register copy
    cpu.push_word(bus, cpu.pc.wrapping_add(1));
    cpu.push_byte(bus, status_copy.bits());

    // Load PC from interrupt vector
    cpu.pc = bus.mem_read_word_page_wrap(0xFFFE);

    inst.cycles
}

fn handle_bvc<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    if !cpu.status.contains(Flags::OVERFLOW) {
        let offset = addr.read(cpu, bus) as i8;
        let old_pc = cpu.pc;
        cpu.pc = cpu.pc.wrapping_add(offset as u16);
        inst.cycles + if (old_pc & 0xFF00) != (cpu.pc & 0xFF00) { 2 } else { 1 }
    } else {
        inst.cycles
    }
}

fn handle_bvs<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    if cpu.status.contains(Flags::OVERFLOW) {
        let offset = addr.read(cpu, bus) as i8;
        let old_pc = cpu.pc;
        cpu.pc = cpu.pc.wrapping_add(offset as u16);
        inst.cycles + if (old_pc & 0xFF00) != (cpu.pc & 0xFF00) { 2 } else { 1 }
    } else {
        inst.cycles
    }
}

fn handle_clc<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.status.remove(Flags::CARRY);
    inst.cycles
}

fn handle_cld<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.status.remove(Flags::DECIMAL);
    inst.cycles
}

fn handle_cli<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.status.remove(Flags::INTERRUPT);
    inst.cycles
}

fn handle_clv<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.status.remove(Flags::OVERFLOW);
    inst.cycles
}

fn handle_cmp<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    let value = addr.read(cpu, bus);
    let result = cpu.a.wrapping_sub(value);

    Flags::set(&mut cpu.status, Flags::CARRY, cpu.a >= value);
    Flags::set(&mut cpu.status, Flags::NEGATIVE, result & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    inst.cycles
}

fn handle_cpx<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    let value = addr.read(cpu, bus);
    let result = cpu.x.wrapping_sub(value);

    Flags::set(&mut cpu.status, Flags::CARRY, cpu.x >= value);
    Flags::set(&mut cpu.status, Flags::NEGATIVE, result & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    inst.cycles
}

fn handle_cpy<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    let value = addr.read(cpu, bus);
    let result = cpu.y.wrapping_sub(value);

    Flags::set(&mut cpu.status, Flags::CARRY, cpu.y >= value);
    Flags::set(&mut cpu.status, Flags::NEGATIVE, result & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    inst.cycles
}

fn handle_dec<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    let value = addr.read(cpu, bus);
    let result = value.wrapping_sub(1);

    Flags::set(&mut cpu.status, Flags::NEGATIVE, result & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    addr.write(cpu, bus, result);
    inst.cycles
}

fn handle_dex<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.x = cpu.x.wrapping_sub(1);

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.x & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.x == 0);

    inst.cycles
}

fn handle_dey<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.y = cpu.y.wrapping_sub(1);

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.y & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.y == 0);

    inst.cycles
}

fn handle_eor<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    let value = addr.read(cpu, bus);
    cpu.a ^= value;

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.a & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.a == 0);

    inst.cycles
}

fn handle_inc<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    let value = addr.read(cpu, bus);
    let result = value.wrapping_add(1);

    Flags::set(&mut cpu.status, Flags::NEGATIVE, result & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    addr.write(cpu, bus, result);
    inst.cycles
}

fn handle_inx<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.x = cpu.x.wrapping_add(1);

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.x & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.x == 0);

    inst.cycles
}

fn handle_iny<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.y = cpu.y.wrapping_add(1);

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.y & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.y == 0);

    inst.cycles
}

fn handle_jmp<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    if let EffectiveAddress::Memory(addr) = addr {
        cpu.pc = addr;
    }
    inst.cycles
}

fn handle_jsr<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    let return_addr = cpu.pc.wrapping_sub(1);
    cpu.push_word(bus, return_addr);
    if let EffectiveAddress::Memory(addr) = addr {
        cpu.pc = addr;
    }
    inst.cycles
}

fn handle_lda<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    cpu.a = addr.read(cpu, bus);

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.a & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.a == 0);

    inst.cycles
}

fn handle_ldx<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    cpu.x = addr.read(cpu, bus);

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.x & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.x == 0);

    inst.cycles
}

fn handle_ldy<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    cpu.y = addr.read(cpu, bus);

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.y & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.y == 0);

    inst.cycles
}

fn handle_lsr<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    let value = addr.read(cpu, bus);
    let result = value >> 1;

    Flags::set(&mut cpu.status, Flags::CARRY, value & 0x01 != 0);
    Flags::set(&mut cpu.status, Flags::NEGATIVE, false);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    match inst.address_mode {
        Mode::Accumulator => cpu.a = result,
        _ => addr.write(cpu, bus, result),
    }

    inst.cycles
}

fn handle_nop<B: Bus>(_cpu: &mut CPU, _bus: &mut B, inst: &Instruction<B>) -> usize {
    inst.cycles
}

fn handle_ora<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    let value = addr.read(cpu, bus);
    cpu.a |= value;

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.a & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.a == 0);

    inst.cycles
}

fn handle_pha<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.push_byte(bus, cpu.a);
    inst.cycles
}

fn handle_php<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let mut status = cpu.status;
    status.insert(Flags::BREAK);
    status.insert(Flags::UNUSED);
    cpu.push_byte(bus, status.bits());
    inst.cycles
}

fn handle_pla<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.a = cpu.pop_byte(bus);
    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.a & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.a == 0);
    inst.cycles
}

fn handle_plp<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let status = cpu.pop_byte(bus);
    cpu.status = Flags::from_bits_truncate(status);
    cpu.status.remove(Flags::BREAK);
    cpu.status.remove(Flags::UNUSED);
    inst.cycles
}

fn handle_rol<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    let value = addr.read(cpu, bus);
    let carry = if cpu.status.contains(Flags::CARRY) { 1 } else { 0 };
    let result = (value << 1) | carry;

    Flags::set(&mut cpu.status, Flags::CARRY, value & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::NEGATIVE, result & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    match inst.address_mode {
        Mode::Accumulator => cpu.a = result,
        _ => addr.write(cpu, bus, result),
    }

    inst.cycles
}

fn handle_ror<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    let value = addr.read(cpu, bus);
    let carry = if cpu.status.contains(Flags::CARRY) { 0x80 } else { 0 };
    let result = (value >> 1) | carry;

    Flags::set(&mut cpu.status, Flags::CARRY, value & 0x01 != 0);
    Flags::set(&mut cpu.status, Flags::NEGATIVE, result & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    match inst.address_mode {
        Mode::Accumulator => cpu.a = result,
        _ => addr.write(cpu, bus, result),
    }

    inst.cycles
}

fn handle_rti<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let status = cpu.pop_byte(bus);
    cpu.status = Flags::from_bits_truncate(status);
    cpu.status.remove(Flags::BREAK);
    cpu.status.remove(Flags::UNUSED);
    cpu.pc = cpu.pop_word(bus).wrapping_add(1);
    inst.cycles
}

fn handle_rts<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.pc = cpu.pop_word(bus).wrapping_add(1);
    inst.cycles
}

fn handle_sbc<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    let value = addr.read(cpu, bus);
    let carry = if cpu.status.contains(Flags::CARRY) { 1 } else { 0 };
    
    // First do binary subtraction
    let binary_result = cpu.a.wrapping_sub(value).wrapping_sub(1 - carry);
    
    // Calculate flags based on binary result
    Flags::set(&mut cpu.status, Flags::CARRY, cpu.a >= value && (cpu.a > value || carry == 1));
    Flags::set(&mut cpu.status, Flags::OVERFLOW, (cpu.a ^ value) & (cpu.a ^ binary_result) & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::NEGATIVE, binary_result & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, binary_result == 0);

    // If in BCD mode, adjust the result
    let result = if cpu.status.contains(Flags::DECIMAL) {
        let mut al = binary_result & 0x0F;
        let mut ah = binary_result >> 4;
        
        if al > 9 {
            al -= 6;
            ah -= 1;
        }
        
        if ah > 9 {
            ah -= 6;
            cpu.status.remove(Flags::CARRY);
        }
        
        ((ah & 0x0F) << 4) | (al & 0x0F)
    } else {
        binary_result
    };

    cpu.a = result;
    inst.cycles
}

fn handle_sec<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.status.insert(Flags::CARRY);
    inst.cycles
}

fn handle_sed<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.status.insert(Flags::DECIMAL);
    inst.cycles
}

fn handle_sei<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.status.insert(Flags::INTERRUPT);
    inst.cycles
}

fn handle_sta<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    addr.write(cpu, bus, cpu.a);
    inst.cycles
}

fn handle_stx<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    addr.write(cpu, bus, cpu.x);
    inst.cycles
}

fn handle_sty<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Instruction<B>) -> usize {
    let addr = inst.address_mode.fetch(cpu, bus);
    addr.write(cpu, bus, cpu.y);
    inst.cycles
}

fn handle_tax<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.x = cpu.a;
    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.x & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.x == 0);
    inst.cycles
}

fn handle_tay<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.y = cpu.a;
    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.y & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.y == 0);
    inst.cycles
}

fn handle_tsx<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.x = cpu.sp;
    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.x & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.x == 0);
    inst.cycles
}

fn handle_txa<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.a = cpu.x;
    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.a & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.a == 0);
    inst.cycles
}

fn handle_txs<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.sp = cpu.x;
    inst.cycles
}

fn handle_tya<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Instruction<B>) -> usize {
    cpu.a = cpu.y;
    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.a & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.a == 0);
    inst.cycles
}