use crate::cpu::w65c02::{cpu::{Flags, VECTOR_IRQ}, Bus, CPU};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Operand {
    Immediate(u8),
    ZeroPage(u8),
    ZeroPageX(u8),
    ZeroPageY(u8),
    Absolute(u16),
    AbsoluteX(u16),
    AbsoluteY(u16),
    Indirect(u16),
    IndirectX(u8),
    IndirectY(u8),
    Relative(u16),
    Accumulator,
    Implied,
}

impl std::fmt::Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Immediate(value) => write!(f, "#${:02X}", value),
            Operand::ZeroPage(addr) => write!(f, "${:02X}", addr),
            Operand::ZeroPageX(addr) => write!(f, "${:02X},X", addr),
            Operand::ZeroPageY(addr) => write!(f, "${:02X},Y", addr),
            Operand::Absolute(addr) => write!(f, "${:04X}", addr),
            Operand::AbsoluteX(addr) => write!(f, "${:04X},X", addr),
            Operand::AbsoluteY(addr) => write!(f, "${:04X},Y", addr),
            Operand::Indirect(addr) => write!(f, "(${:04X})", addr),
            Operand::IndirectX(addr) => write!(f, "(${:02X},X)", addr),
            Operand::IndirectY(addr) => write!(f, "(${:02X}),Y", addr),
            Operand::Relative(addr) => write!(f, "${:04X}", addr),
            Operand::Accumulator => write!(f, "A"),
            Operand::Implied => write!(f, ""),
        }
    }
}

impl Operand {
    pub fn read(&self, cpu: &mut CPU, bus: &mut impl Bus) -> u8 {
        match self {
            Operand::Immediate(value) => *value,
            Operand::ZeroPage(addr) => bus.mem_read(*addr as u16),
            Operand::ZeroPageX(addr) => bus.mem_read(addr.wrapping_add(cpu.x) as u16),
            Operand::ZeroPageY(addr) => bus.mem_read(addr.wrapping_add(cpu.y) as u16),
            Operand::Absolute(addr) => bus.mem_read(*addr),
            Operand::AbsoluteX(addr) => bus.mem_read(addr.wrapping_add(cpu.x as u16)),
            Operand::AbsoluteY(addr) => bus.mem_read(addr.wrapping_add(cpu.y as u16)),
            Operand::Indirect(addr) => {
                let ptr = bus.mem_read_word(*addr);
                bus.mem_read(ptr)
            },
            Operand::IndirectX(addr) => {
                let ptr = bus.mem_read_word((*addr as u16).wrapping_add(cpu.x as u16));
                bus.mem_read(ptr)
            },
            Operand::IndirectY(addr) => {
                let ptr = bus.mem_read_word(*addr as u16);
                bus.mem_read(ptr.wrapping_add(cpu.y as u16))
            },
            Operand::Relative(addr) => bus.mem_read(*addr),
            Operand::Accumulator => cpu.a,
            Operand::Implied => panic!("read of implied operand"),
        }
    }

    pub fn write(&self, cpu: &mut CPU, bus: &mut impl Bus, val: u8) {
        match self {
            Operand::ZeroPage(addr) => bus.mem_write(*addr as u16, val),
            Operand::ZeroPageX(addr) => bus.mem_write(addr.wrapping_add(cpu.x) as u16, val),
            Operand::ZeroPageY(addr) => bus.mem_write(addr.wrapping_add(cpu.y) as u16, val),
            Operand::Absolute(addr) => bus.mem_write(*addr, val),
            Operand::AbsoluteX(addr) => bus.mem_write(addr.wrapping_add(cpu.x as u16), val),
            Operand::AbsoluteY(addr) => bus.mem_write(addr.wrapping_add(cpu.y as u16), val),
            Operand::Indirect(addr) => {
                let ptr = bus.mem_read_word(*addr);
                bus.mem_write(ptr, val)
            },
            Operand::IndirectX(addr) => {
                let ptr = bus.mem_read_word((*addr as u16).wrapping_add(cpu.x as u16));
                bus.mem_write(ptr, val)
            },
            Operand::IndirectY(addr) => {
                let ptr = bus.mem_read_word(*addr as u16);
                bus.mem_write(ptr.wrapping_add(cpu.y as u16), val)
            },
            Operand::Accumulator => cpu.a = val,
            _ => panic!("write of invalid operand"),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Operand::Immediate(_) => 2,
            Operand::ZeroPage(_) => 2,
            Operand::ZeroPageX(_) => 2,
            Operand::ZeroPageY(_) => 2,
            Operand::Absolute(_) => 3,
            Operand::AbsoluteX(_) => 3,
            Operand::AbsoluteY(_) => 3,
            Operand::Indirect(_) => 3,
            Operand::IndirectX(_) => 2,
            Operand::IndirectY(_) => 2,
            Operand::Relative(_) => 2,
            Operand::Accumulator => 0,
            Operand::Implied => 0,
        }
    }
}

pub struct Instruction {
    pub opcode: Opcode,
    pub operand: Operand,
    pub cycles: usize,
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {}", self.opcode, self.operand)
    }
}

impl Instruction {
    pub fn len(&self) -> usize {
        self.operand.len() + 1
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    Absolute,       // $4400
    AbsoluteX,      // $4400,X
    AbsoluteY,      // $4400,Y
    Accumulator,    // A
    Immediate,      // #$44
    Implied,        //
    Indirect,       // ($4400)
    IndirectX,      // ($44,X)
    IndirectY,      // ($44),Y
    Relative,       // 
    ZeroPage,       // $44
    ZeroPageX,      // $44,X
    ZeroPageY,      // $44,Y
}

impl Mode {
    pub fn fetch(&self, cpu: &mut CPU, bus: &mut impl Bus) -> Operand {
        match self {
            Mode::Immediate => Operand::Immediate(cpu.fetch_byte(bus)),
            Mode::ZeroPage => Operand::ZeroPage(cpu.fetch_byte(bus)),
            Mode::ZeroPageX => Operand::ZeroPageX(cpu.fetch_byte(bus)),
            Mode::ZeroPageY => Operand::ZeroPageY(cpu.fetch_byte(bus)),
            Mode::Absolute => Operand::Absolute(cpu.fetch_word(bus)),
            Mode::AbsoluteX => Operand::AbsoluteX(cpu.fetch_word(bus)),
            Mode::AbsoluteY => Operand::AbsoluteY(cpu.fetch_word(bus)),
            Mode::Indirect => Operand::Indirect(cpu.fetch_word(bus)),
            Mode::IndirectX => Operand::IndirectX(cpu.fetch_byte(bus)),
            Mode::IndirectY => Operand::IndirectY(cpu.fetch_byte(bus)),
            Mode::Relative => {
                let offset = cpu.fetch_byte(bus) as i8;
                Operand::Relative(cpu.pc.wrapping_add(offset as u16))
            },
            Mode::Accumulator => Operand::Accumulator,
            Mode::Implied => Operand::Implied,
        }
    }
}

/// An instruction handler takes a CPU, a bus, and an instruction, and returns the number of cycles 
/// taken to execute the instruction.
pub type Handler<B> = fn(&mut CPU, &mut B, &Decoded<B>) -> Instruction;


pub struct Decoded<B: Bus> {
    pub opcode: Opcode,
    pub cycles: usize,
    pub address_mode: Mode,
    pub handler: Handler<B>,
}

pub fn decode<B: Bus>(cpu: &mut CPU, bus: &mut B) -> Decoded<B> {
    match cpu.fetch_byte(bus) {
        // ADC
        0x69 => Decoded { opcode: Opcode::ADC, cycles: 2, address_mode: Mode::Immediate, handler: handle_adc },
        0x65 => Decoded { opcode: Opcode::ADC, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_adc },
        0x75 => Decoded { opcode: Opcode::ADC, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_adc },
        0x6D => Decoded { opcode: Opcode::ADC, cycles: 4, address_mode: Mode::Absolute, handler: handle_adc },
        0x7D => Decoded { opcode: Opcode::ADC, cycles: 4, address_mode: Mode::AbsoluteX, handler: handle_adc },
        0x79 => Decoded { opcode: Opcode::ADC, cycles: 4, address_mode: Mode::AbsoluteY, handler: handle_adc },
        0x61 => Decoded { opcode: Opcode::ADC, cycles: 6, address_mode: Mode::IndirectX, handler: handle_adc },
        0x71 => Decoded { opcode: Opcode::ADC, cycles: 5, address_mode: Mode::IndirectY, handler: handle_adc },

        // AND
        0x29 => Decoded { opcode: Opcode::AND, cycles: 2, address_mode: Mode::Immediate, handler: handle_and },
        0x25 => Decoded { opcode: Opcode::AND, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_and },
        0x35 => Decoded { opcode: Opcode::AND, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_and },
        0x2D => Decoded { opcode: Opcode::AND, cycles: 4, address_mode: Mode::Absolute, handler: handle_and },
        0x3D => Decoded { opcode: Opcode::AND, cycles: 4, address_mode: Mode::AbsoluteX, handler: handle_and },
        0x39 => Decoded { opcode: Opcode::AND, cycles: 4, address_mode: Mode::AbsoluteY, handler: handle_and },
        0x21 => Decoded { opcode: Opcode::AND, cycles: 6, address_mode: Mode::IndirectX, handler: handle_and },
        0x31 => Decoded { opcode: Opcode::AND, cycles: 5, address_mode: Mode::IndirectY, handler: handle_and },

        // ASL
        0x0A => Decoded { opcode: Opcode::ASL, cycles: 2, address_mode: Mode::Accumulator, handler: handle_asl },
        0x06 => Decoded { opcode: Opcode::ASL, cycles: 5, address_mode: Mode::ZeroPage, handler: handle_asl },
        0x16 => Decoded { opcode: Opcode::ASL, cycles: 6, address_mode: Mode::ZeroPageX, handler: handle_asl },
        0x0E => Decoded { opcode: Opcode::ASL, cycles: 6, address_mode: Mode::Absolute, handler: handle_asl },
        0x1E => Decoded { opcode: Opcode::ASL, cycles: 7, address_mode: Mode::AbsoluteX, handler: handle_asl },

        // Branch instructions
        0x90 => Decoded { opcode: Opcode::BCC, cycles: 2, address_mode: Mode::Relative, handler: handle_bcc },
        0xB0 => Decoded { opcode: Opcode::BCS, cycles: 2, address_mode: Mode::Relative, handler: handle_bcs },
        0xF0 => Decoded { opcode: Opcode::BEQ, cycles: 2, address_mode: Mode::Relative, handler: handle_beq },
        0x30 => Decoded { opcode: Opcode::BMI, cycles: 2, address_mode: Mode::Relative, handler: handle_bmi },
        0xD0 => Decoded { opcode: Opcode::BNE, cycles: 2, address_mode: Mode::Relative, handler: handle_bne },
        0x10 => Decoded { opcode: Opcode::BPL, cycles: 2, address_mode: Mode::Relative, handler: handle_bpl },
        0x50 => Decoded { opcode: Opcode::BVC, cycles: 2, address_mode: Mode::Relative, handler: handle_bvc },
        0x70 => Decoded { opcode: Opcode::BVS, cycles: 2, address_mode: Mode::Relative, handler: handle_bvs },

        // BIT
        0x24 => Decoded { opcode: Opcode::BIT, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_bit },
        0x2C => Decoded { opcode: Opcode::BIT, cycles: 4, address_mode: Mode::Absolute, handler: handle_bit },

        // BRK
        0x00 => Decoded { opcode: Opcode::BRK, cycles: 7, address_mode: Mode::Implied, handler: handle_brk },

        // Clear flags
        0x18 => Decoded { opcode: Opcode::CLC, cycles: 2, address_mode: Mode::Implied, handler: handle_clc },
        0xD8 => Decoded { opcode: Opcode::CLD, cycles: 2, address_mode: Mode::Implied, handler: handle_cld },
        0x58 => Decoded { opcode: Opcode::CLI, cycles: 2, address_mode: Mode::Implied, handler: handle_cli },
        0xB8 => Decoded { opcode: Opcode::CLV, cycles: 2, address_mode: Mode::Implied, handler: handle_clv },

        // Compare
        0xC9 => Decoded { opcode: Opcode::CMP, cycles: 2, address_mode: Mode::Immediate, handler: handle_cmp },
        0xC5 => Decoded { opcode: Opcode::CMP, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_cmp },
        0xD5 => Decoded { opcode: Opcode::CMP, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_cmp },
        0xCD => Decoded { opcode: Opcode::CMP, cycles: 4, address_mode: Mode::Absolute, handler: handle_cmp },
        0xDD => Decoded { opcode: Opcode::CMP, cycles: 4, address_mode: Mode::AbsoluteX, handler: handle_cmp },
        0xD9 => Decoded { opcode: Opcode::CMP, cycles: 4, address_mode: Mode::AbsoluteY, handler: handle_cmp },
        0xC1 => Decoded { opcode: Opcode::CMP, cycles: 6, address_mode: Mode::IndirectX, handler: handle_cmp },
        0xD1 => Decoded { opcode: Opcode::CMP, cycles: 5, address_mode: Mode::IndirectY, handler: handle_cmp },

        // CPX
        0xE0 => Decoded { opcode: Opcode::CPX, cycles: 2, address_mode: Mode::Immediate, handler: handle_cpx },
        0xE4 => Decoded { opcode: Opcode::CPX, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_cpx },
        0xEC => Decoded { opcode: Opcode::CPX, cycles: 4, address_mode: Mode::Absolute, handler: handle_cpx },

        // CPY
        0xC0 => Decoded { opcode: Opcode::CPY, cycles: 2, address_mode: Mode::Immediate, handler: handle_cpy },
        0xC4 => Decoded { opcode: Opcode::CPY, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_cpy },
        0xCC => Decoded { opcode: Opcode::CPY, cycles: 4, address_mode: Mode::Absolute, handler: handle_cpy },

        // DEC
        0xC6 => Decoded { opcode: Opcode::DEC, cycles: 5, address_mode: Mode::ZeroPage, handler: handle_dec },
        0xD6 => Decoded { opcode: Opcode::DEC, cycles: 6, address_mode: Mode::ZeroPageX, handler: handle_dec },
        0xCE => Decoded { opcode: Opcode::DEC, cycles: 6, address_mode: Mode::Absolute, handler: handle_dec },
        0xDE => Decoded { opcode: Opcode::DEC, cycles: 7, address_mode: Mode::AbsoluteX, handler: handle_dec },

        // Decrement registers
        0xCA => Decoded { opcode: Opcode::DEX, cycles: 2, address_mode: Mode::Implied, handler: handle_dex },
        0x88 => Decoded { opcode: Opcode::DEY, cycles: 2, address_mode: Mode::Implied, handler: handle_dey },

        // EOR
        0x49 => Decoded { opcode: Opcode::EOR, cycles: 2, address_mode: Mode::Immediate, handler: handle_eor },
        0x45 => Decoded { opcode: Opcode::EOR, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_eor },
        0x55 => Decoded { opcode: Opcode::EOR, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_eor },
        0x4D => Decoded { opcode: Opcode::EOR, cycles: 4, address_mode: Mode::Absolute, handler: handle_eor },
        0x5D => Decoded { opcode: Opcode::EOR, cycles: 4, address_mode: Mode::AbsoluteX, handler: handle_eor },
        0x59 => Decoded { opcode: Opcode::EOR, cycles: 4, address_mode: Mode::AbsoluteY, handler: handle_eor },
        0x41 => Decoded { opcode: Opcode::EOR, cycles: 6, address_mode: Mode::IndirectX, handler: handle_eor },
        0x51 => Decoded { opcode: Opcode::EOR, cycles: 5, address_mode: Mode::IndirectY, handler: handle_eor },

        // INC
        0xE6 => Decoded { opcode: Opcode::INC, cycles: 5, address_mode: Mode::ZeroPage, handler: handle_inc },
        0xF6 => Decoded { opcode: Opcode::INC, cycles: 6, address_mode: Mode::ZeroPageX, handler: handle_inc },
        0xEE => Decoded { opcode: Opcode::INC, cycles: 6, address_mode: Mode::Absolute, handler: handle_inc },
        0xFE => Decoded { opcode: Opcode::INC, cycles: 7, address_mode: Mode::AbsoluteX, handler: handle_inc },

        // Increment registers
        0xE8 => Decoded { opcode: Opcode::INX, cycles: 2, address_mode: Mode::Implied, handler: handle_inx },
        0xC8 => Decoded { opcode: Opcode::INY, cycles: 2, address_mode: Mode::Implied, handler: handle_iny },

        // JMP
        0x4C => Decoded { opcode: Opcode::JMP, cycles: 3, address_mode: Mode::Absolute, handler: handle_jmp },
        0x6C => Decoded { opcode: Opcode::JMP, cycles: 5, address_mode: Mode::Indirect, handler: handle_jmp },

        // JSR
        0x20 => Decoded { opcode: Opcode::JSR, cycles: 6, address_mode: Mode::Absolute, handler: handle_jsr },

        // LDA
        0xA9 => Decoded { opcode: Opcode::LDA, cycles: 2, address_mode: Mode::Immediate, handler: handle_lda },
        0xA5 => Decoded { opcode: Opcode::LDA, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_lda },
        0xB5 => Decoded { opcode: Opcode::LDA, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_lda },
        0xAD => Decoded { opcode: Opcode::LDA, cycles: 4, address_mode: Mode::Absolute, handler: handle_lda },
        0xBD => Decoded { opcode: Opcode::LDA, cycles: 4, address_mode: Mode::AbsoluteX, handler: handle_lda },
        0xB9 => Decoded { opcode: Opcode::LDA, cycles: 4, address_mode: Mode::AbsoluteY, handler: handle_lda },
        0xA1 => Decoded { opcode: Opcode::LDA, cycles: 6, address_mode: Mode::IndirectX, handler: handle_lda },
        0xB1 => Decoded { opcode: Opcode::LDA, cycles: 5, address_mode: Mode::IndirectY, handler: handle_lda },

        // LDX
        0xA2 => Decoded { opcode: Opcode::LDX, cycles: 2, address_mode: Mode::Immediate, handler: handle_ldx },
        0xA6 => Decoded { opcode: Opcode::LDX, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_ldx },
        0xB6 => Decoded { opcode: Opcode::LDX, cycles: 4, address_mode: Mode::ZeroPageY, handler: handle_ldx },
        0xAE => Decoded { opcode: Opcode::LDX, cycles: 4, address_mode: Mode::Absolute, handler: handle_ldx },
        0xBE => Decoded { opcode: Opcode::LDX, cycles: 4, address_mode: Mode::AbsoluteY, handler: handle_ldx },

        // LDY
        0xA0 => Decoded { opcode: Opcode::LDY, cycles: 2, address_mode: Mode::Immediate, handler: handle_ldy },
        0xA4 => Decoded { opcode: Opcode::LDY, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_ldy },
        0xB4 => Decoded { opcode: Opcode::LDY, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_ldy },
        0xAC => Decoded { opcode: Opcode::LDY, cycles: 4, address_mode: Mode::Absolute, handler: handle_ldy },
        0xBC => Decoded { opcode: Opcode::LDY, cycles: 4, address_mode: Mode::AbsoluteX, handler: handle_ldy },

        // LSR
        0x4A => Decoded { opcode: Opcode::LSR, cycles: 2, address_mode: Mode::Accumulator, handler: handle_lsr },
        0x46 => Decoded { opcode: Opcode::LSR, cycles: 5, address_mode: Mode::ZeroPage, handler: handle_lsr },
        0x56 => Decoded { opcode: Opcode::LSR, cycles: 6, address_mode: Mode::ZeroPageX, handler: handle_lsr },
        0x4E => Decoded { opcode: Opcode::LSR, cycles: 6, address_mode: Mode::Absolute, handler: handle_lsr },
        0x5E => Decoded { opcode: Opcode::LSR, cycles: 7, address_mode: Mode::AbsoluteX, handler: handle_lsr },

        // NOP
        0xEA => Decoded { opcode: Opcode::NOP, cycles: 2, address_mode: Mode::Implied, handler: handle_nop },

        // ORA
        0x09 => Decoded { opcode: Opcode::ORA, cycles: 2, address_mode: Mode::Immediate, handler: handle_ora },
        0x05 => Decoded { opcode: Opcode::ORA, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_ora },
        0x15 => Decoded { opcode: Opcode::ORA, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_ora },
        0x0D => Decoded { opcode: Opcode::ORA, cycles: 4, address_mode: Mode::Absolute, handler: handle_ora },
        0x1D => Decoded { opcode: Opcode::ORA, cycles: 4, address_mode: Mode::AbsoluteX, handler: handle_ora },
        0x19 => Decoded { opcode: Opcode::ORA, cycles: 4, address_mode: Mode::AbsoluteY, handler: handle_ora },
        0x01 => Decoded { opcode: Opcode::ORA, cycles: 6, address_mode: Mode::IndirectX, handler: handle_ora },
        0x11 => Decoded { opcode: Opcode::ORA, cycles: 5, address_mode: Mode::IndirectY, handler: handle_ora },

        // Stack operations
        0x48 => Decoded { opcode: Opcode::PHA, cycles: 3, address_mode: Mode::Implied, handler: handle_pha },
        0x08 => Decoded { opcode: Opcode::PHP, cycles: 3, address_mode: Mode::Implied, handler: handle_php },
        0x68 => Decoded { opcode: Opcode::PLA, cycles: 4, address_mode: Mode::Implied, handler: handle_pla },
        0x28 => Decoded { opcode: Opcode::PLP, cycles: 4, address_mode: Mode::Implied, handler: handle_plp },

        // ROL
        0x2A => Decoded { opcode: Opcode::ROL, cycles: 2, address_mode: Mode::Accumulator, handler: handle_rol },
        0x26 => Decoded { opcode: Opcode::ROL, cycles: 5, address_mode: Mode::ZeroPage, handler: handle_rol },
        0x36 => Decoded { opcode: Opcode::ROL, cycles: 6, address_mode: Mode::ZeroPageX, handler: handle_rol },
        0x2E => Decoded { opcode: Opcode::ROL, cycles: 6, address_mode: Mode::Absolute, handler: handle_rol },
        0x3E => Decoded { opcode: Opcode::ROL, cycles: 7, address_mode: Mode::AbsoluteX, handler: handle_rol },

        // ROR
        0x6A => Decoded { opcode: Opcode::ROR, cycles: 2, address_mode: Mode::Accumulator, handler: handle_ror },
        0x66 => Decoded { opcode: Opcode::ROR, cycles: 5, address_mode: Mode::ZeroPage, handler: handle_ror },
        0x76 => Decoded { opcode: Opcode::ROR, cycles: 6, address_mode: Mode::ZeroPageX, handler: handle_ror },
        0x6E => Decoded { opcode: Opcode::ROR, cycles: 6, address_mode: Mode::Absolute, handler: handle_ror },
        0x7E => Decoded { opcode: Opcode::ROR, cycles: 7, address_mode: Mode::AbsoluteX, handler: handle_ror },

        // RTI
        0x40 => Decoded { opcode: Opcode::RTI, cycles: 6, address_mode: Mode::Implied, handler: handle_rti },

        // RTS
        0x60 => Decoded { opcode: Opcode::RTS, cycles: 6, address_mode: Mode::Implied, handler: handle_rts },

        // SBC
        0xE9 => Decoded { opcode: Opcode::SBC, cycles: 2, address_mode: Mode::Immediate, handler: handle_sbc },
        0xE5 => Decoded { opcode: Opcode::SBC, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_sbc },
        0xF5 => Decoded { opcode: Opcode::SBC, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_sbc },
        0xED => Decoded { opcode: Opcode::SBC, cycles: 4, address_mode: Mode::Absolute, handler: handle_sbc },
        0xFD => Decoded { opcode: Opcode::SBC, cycles: 4, address_mode: Mode::AbsoluteX, handler: handle_sbc },
        0xF9 => Decoded { opcode: Opcode::SBC, cycles: 4, address_mode: Mode::AbsoluteY, handler: handle_sbc },
        0xE1 => Decoded { opcode: Opcode::SBC, cycles: 6, address_mode: Mode::IndirectX, handler: handle_sbc },
        0xF1 => Decoded { opcode: Opcode::SBC, cycles: 5, address_mode: Mode::IndirectY, handler: handle_sbc },

        // Set flags
        0x38 => Decoded { opcode: Opcode::SEC, cycles: 2, address_mode: Mode::Implied, handler: handle_sec },
        0xF8 => Decoded { opcode: Opcode::SED, cycles: 2, address_mode: Mode::Implied, handler: handle_sed },
        0x78 => Decoded { opcode: Opcode::SEI, cycles: 2, address_mode: Mode::Implied, handler: handle_sei },

        // Store instructions
        0x85 => Decoded { opcode: Opcode::STA, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_sta },
        0x95 => Decoded { opcode: Opcode::STA, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_sta },
        0x8D => Decoded { opcode: Opcode::STA, cycles: 4, address_mode: Mode::Absolute, handler: handle_sta },
        0x9D => Decoded { opcode: Opcode::STA, cycles: 5, address_mode: Mode::AbsoluteX, handler: handle_sta },
        0x99 => Decoded { opcode: Opcode::STA, cycles: 5, address_mode: Mode::AbsoluteY, handler: handle_sta },
        0x81 => Decoded { opcode: Opcode::STA, cycles: 6, address_mode: Mode::IndirectX, handler: handle_sta },
        0x91 => Decoded { opcode: Opcode::STA, cycles: 6, address_mode: Mode::IndirectY, handler: handle_sta },

        0x86 => Decoded { opcode: Opcode::STX, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_stx },
        0x96 => Decoded { opcode: Opcode::STX, cycles: 4, address_mode: Mode::ZeroPageY, handler: handle_stx },
        0x8E => Decoded { opcode: Opcode::STX, cycles: 4, address_mode: Mode::Absolute, handler: handle_stx },

        0x84 => Decoded { opcode: Opcode::STY, cycles: 3, address_mode: Mode::ZeroPage, handler: handle_sty },
        0x94 => Decoded { opcode: Opcode::STY, cycles: 4, address_mode: Mode::ZeroPageX, handler: handle_sty },
        0x8C => Decoded { opcode: Opcode::STY, cycles: 4, address_mode: Mode::Absolute, handler: handle_sty },

        // Transfer instructions
        0xAA => Decoded { opcode: Opcode::TAX, cycles: 2, address_mode: Mode::Implied, handler: handle_tax },
        0xA8 => Decoded { opcode: Opcode::TAY, cycles: 2, address_mode: Mode::Implied, handler: handle_tay },
        0xBA => Decoded { opcode: Opcode::TSX, cycles: 2, address_mode: Mode::Implied, handler: handle_tsx },
        0x8A => Decoded { opcode: Opcode::TXA, cycles: 2, address_mode: Mode::Implied, handler: handle_txa },
        0x9A => Decoded { opcode: Opcode::TXS, cycles: 2, address_mode: Mode::Implied, handler: handle_txs },
        0x98 => Decoded { opcode: Opcode::TYA, cycles: 2, address_mode: Mode::Implied, handler: handle_tya },

        _ => panic!("Invalid opcode: {:02X}", cpu.pc - 1),
    }
}

// Handler stubs for each opcode
fn handle_adc<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let value = operand.read(cpu, bus);
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

    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_and<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let value = operand.read(cpu, bus);
    cpu.a &= value;

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.a & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.a == 0);

    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_asl<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let value = operand.read(cpu, bus);
    let result = value << 1;

    Flags::set(&mut cpu.status, Flags::CARRY, value & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::NEGATIVE, result & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    match inst.address_mode {
        Mode::Accumulator => cpu.a = result,
        _ => operand.write(cpu, bus, result),
    }

    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_bcc<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let mut cycles = inst.cycles;
    if !cpu.status.contains(Flags::CARRY) {
        if let Operand::Relative(addr) = operand {
            let old_pc = cpu.pc;
            cpu.pc = addr;
            cycles += if (old_pc & 0xFF00) != (cpu.pc & 0xFF00) { 2 } else { 1 };
        }
    }
    Instruction {
        opcode: inst.opcode,
        operand,
        cycles,
    }
}

fn handle_bcs<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let mut cycles = inst.cycles;
    if cpu.status.contains(Flags::CARRY) {
        if let Operand::Relative(addr) = operand {
            let old_pc = cpu.pc;
            cpu.pc = addr;
            cycles += if (old_pc & 0xFF00) != (cpu.pc & 0xFF00) { 2 } else { 1 };
        }
    }
    Instruction {
        opcode: inst.opcode,
        operand,
        cycles,
    }
}

fn handle_beq<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let mut cycles = inst.cycles;
    if cpu.status.contains(Flags::ZERO) {
        if let Operand::Relative(addr) = operand {
            let old_pc = cpu.pc;
            cpu.pc = addr;
            cycles += if (old_pc & 0xFF00) != (cpu.pc & 0xFF00) { 2 } else { 1 };
        }
    }
    Instruction {
        opcode: inst.opcode,
        operand,
        cycles,
    }
}

fn handle_bmi<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let mut cycles = inst.cycles;
    if cpu.status.contains(Flags::NEGATIVE) {
        if let Operand::Relative(addr) = operand {
            let old_pc = cpu.pc;
            cpu.pc = addr;
            cycles += if (old_pc & 0xFF00) != (cpu.pc & 0xFF00) { 2 } else { 1 };
        }
    }
    Instruction {
        opcode: inst.opcode,
        operand,
        cycles,
    }
}

fn handle_bne<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let mut cycles = inst.cycles;
    if !cpu.status.contains(Flags::ZERO) {
        if let Operand::Relative(addr) = operand {
            let old_pc = cpu.pc;
            cpu.pc = addr;
            cycles += if (old_pc & 0xFF00) != (cpu.pc & 0xFF00) { 2 } else { 1 };
        }
    }
    Instruction {
        opcode: inst.opcode,
        operand,
        cycles,
    }
}

fn handle_bpl<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let mut cycles = inst.cycles;
    if !cpu.status.contains(Flags::NEGATIVE) {
        if let Operand::Relative(addr) = operand {
            let old_pc = cpu.pc;
            cpu.pc = addr;
            cycles += if (old_pc & 0xFF00) != (cpu.pc & 0xFF00) { 2 } else { 1 };
        }
    }
    Instruction {
        opcode: inst.opcode,
        operand,
        cycles,
    }
}

fn handle_bvc<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let mut cycles = inst.cycles;
    if !cpu.status.contains(Flags::OVERFLOW) {
        if let Operand::Relative(addr) = operand {
            let old_pc = cpu.pc;
            cpu.pc = addr;
            cycles += if (old_pc & 0xFF00) != (cpu.pc & 0xFF00) { 2 } else { 1 };
        }
    }
    Instruction {
        opcode: inst.opcode,
        operand,
        cycles,
    }
}

fn handle_bvs<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let mut cycles = inst.cycles;
    if cpu.status.contains(Flags::OVERFLOW) {
        if let Operand::Relative(addr) = operand {
            let old_pc = cpu.pc;
            cpu.pc = addr;
            cycles += if (old_pc & 0xFF00) != (cpu.pc & 0xFF00) { 2 } else { 1 };
        }
    }
    Instruction {
        opcode: inst.opcode,
        operand,
        cycles,
    }
}

fn handle_brk<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.pc += 1;
    cpu.push_word(bus, cpu.pc);
    let mut status = cpu.status;
    status.insert(Flags::BREAK);
    status.insert(Flags::UNUSED);
    cpu.push_byte(bus, status.bits());
    cpu.status.insert(Flags::INTERRUPT);
    cpu.pc = bus.mem_read_word(VECTOR_IRQ);

    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_clc<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.status.remove(Flags::CARRY);
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_cld<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.status.remove(Flags::DECIMAL);
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_cli<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.status.remove(Flags::INTERRUPT);
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_clv<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.status.remove(Flags::OVERFLOW);
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_cmp<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let value = operand.read(cpu, bus);
    let result = cpu.a.wrapping_sub(value);

    Flags::set(&mut cpu.status, Flags::CARRY, cpu.a >= value);
    Flags::set(&mut cpu.status, Flags::NEGATIVE, result & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_cpx<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let value = operand.read(cpu, bus);
    let result = cpu.x.wrapping_sub(value);

    Flags::set(&mut cpu.status, Flags::CARRY, cpu.x >= value);
    Flags::set(&mut cpu.status, Flags::NEGATIVE, result & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_cpy<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let value = operand.read(cpu, bus);
    let result = cpu.y.wrapping_sub(value);

    Flags::set(&mut cpu.status, Flags::CARRY, cpu.y >= value);
    Flags::set(&mut cpu.status, Flags::NEGATIVE, result & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_dec<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let value = operand.read(cpu, bus);
    let result = value.wrapping_sub(1);

    Flags::set(&mut cpu.status, Flags::NEGATIVE, result & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    operand.write(cpu, bus, result);
    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_dex<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.x = cpu.x.wrapping_sub(1);

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.x & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.x == 0);

    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_dey<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.y = cpu.y.wrapping_sub(1);

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.y & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.y == 0);

    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_eor<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let value = operand.read(cpu, bus);
    cpu.a ^= value;

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.a & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.a == 0);

    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_inc<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let value = operand.read(cpu, bus);
    let result = value.wrapping_add(1);

    Flags::set(&mut cpu.status, Flags::NEGATIVE, result & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    operand.write(cpu, bus, result);
    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_inx<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.x = cpu.x.wrapping_add(1);

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.x & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.x == 0);

    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_iny<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.y = cpu.y.wrapping_add(1);

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.y & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.y == 0);

    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_jmp<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    match operand {
        Operand::Absolute(addr) => cpu.pc = addr,
        Operand::Indirect(addr) => cpu.pc = bus.mem_read_word_page_wrap(addr),
        _ => panic!("Invalid operand for JMP"),
    }
    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_jsr<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    if let Operand::Absolute(addr) = operand {
        cpu.push_word(bus, cpu.pc - 1);
        cpu.pc = addr;
    }
    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_lda<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let value = operand.read(cpu, bus);
    cpu.a = value;

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.a & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.a == 0);

    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_ldx<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let value = operand.read(cpu, bus);
    cpu.x = value;

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.x & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.x == 0);

    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_ldy<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let value = operand.read(cpu, bus);
    cpu.y = value;

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.y & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.y == 0);

    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_lsr<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let value = operand.read(cpu, bus);
    let result = value >> 1;

    Flags::set(&mut cpu.status, Flags::CARRY, value & 0x01 != 0);
    Flags::set(&mut cpu.status, Flags::NEGATIVE, false);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    match inst.address_mode {
        Mode::Accumulator => cpu.a = result,
        _ => operand.write(cpu, bus, result),
    }

    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_nop<B: Bus>(_cpu: &mut CPU, _bus: &mut B, inst: &Decoded<B>) -> Instruction {
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_ora<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let value = operand.read(cpu, bus);
    cpu.a |= value;

    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.a & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.a == 0);

    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_pha<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.push_byte(bus, cpu.a);
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_php<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let mut status = cpu.status;
    status.insert(Flags::BREAK);
    status.insert(Flags::UNUSED);
    cpu.push_byte(bus, status.bits());
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_pla<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.a = cpu.pop_byte(bus);
    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.a & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.a == 0);
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_plp<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let status = cpu.pop_byte(bus);
    cpu.status = Flags::from_bits_truncate(status);
    cpu.status.remove(Flags::BREAK);
    cpu.status.remove(Flags::UNUSED);
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_rol<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let value = operand.read(cpu, bus);
    let carry = if cpu.status.contains(Flags::CARRY) { 1 } else { 0 };
    let result = (value << 1) | carry;

    Flags::set(&mut cpu.status, Flags::CARRY, value & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::NEGATIVE, result & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    match inst.address_mode {
        Mode::Accumulator => cpu.a = result,
        _ => operand.write(cpu, bus, result),
    }

    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_ror<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let value = operand.read(cpu, bus);
    let carry = if cpu.status.contains(Flags::CARRY) { 0x80 } else { 0 };
    let result = (value >> 1) | carry;

    Flags::set(&mut cpu.status, Flags::CARRY, value & 0x01 != 0);
    Flags::set(&mut cpu.status, Flags::NEGATIVE, result & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    match inst.address_mode {
        Mode::Accumulator => cpu.a = result,
        _ => operand.write(cpu, bus, result),
    }

    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_rti<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let status = cpu.pop_byte(bus);
    cpu.status = Flags::from_bits_truncate(status);
    cpu.status.remove(Flags::BREAK);
    cpu.status.remove(Flags::UNUSED);
    cpu.pc = cpu.pop_word(bus).wrapping_add(1);
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: 6,
    }
}

fn handle_rts<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.pc = cpu.pop_word(bus).wrapping_add(1);
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: 6,
    }
}

fn handle_sbc<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let value = operand.read(cpu, bus);
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
    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_sec<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.status.insert(Flags::CARRY);
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_sed<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.status.insert(Flags::DECIMAL);
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_sei<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.status.insert(Flags::INTERRUPT);
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_sta<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    operand.write(cpu, bus, cpu.a);
    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_stx<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    operand.write(cpu, bus, cpu.x);
    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_sty<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    operand.write(cpu, bus, cpu.y);
    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}

fn handle_tax<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.x = cpu.a;
    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.x & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.x == 0);
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_tay<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.y = cpu.a;
    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.y & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.y == 0);
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_tsx<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.x = cpu.sp;
    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.x & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.x == 0);
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_txa<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.a = cpu.x;
    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.a & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.a == 0);
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_txs<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.sp = cpu.x;
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_tya<B: Bus>(cpu: &mut CPU, _bus: &mut B, inst: &Decoded<B>) -> Instruction {
    cpu.a = cpu.y;
    Flags::set(&mut cpu.status, Flags::NEGATIVE, cpu.a & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, cpu.a == 0);
    Instruction {
        opcode: inst.opcode,
        operand: Operand::Implied,
        cycles: inst.cycles,
    }
}

fn handle_bit<B: Bus>(cpu: &mut CPU, bus: &mut B, inst: &Decoded<B>) -> Instruction {
    let operand = inst.address_mode.fetch(cpu, bus);
    let value = operand.read(cpu, bus);
    let result = cpu.a & value;

    Flags::set(&mut cpu.status, Flags::NEGATIVE, value & 0x80 != 0);
    Flags::set(&mut cpu.status, Flags::OVERFLOW, value & 0x40 != 0);
    Flags::set(&mut cpu.status, Flags::ZERO, result == 0);

    Instruction {
        opcode: inst.opcode,
        operand,
        cycles: inst.cycles,
    }
}