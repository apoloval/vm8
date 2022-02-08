use crate::cpu::z81::bus::Bus;
use crate::cpu::z81::reg::Registers;
use crate::cpu::z81::flag::{self, Predicate};
use crate::cpu::z81::op;

pub struct CPU {
    regs: Registers,
    cycles: usize,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            regs: Registers::new(),
            cycles: 0,
        }
    }

    pub fn cycles(&self) -> usize { self.cycles }

    pub fn reset_cycles(&mut self) { self.cycles = 0 }

    pub fn exec<B: Bus>(&mut self, b: &mut B) {
        let opcode = b.mem_read(self.regs.pc());
        self.decode(b, opcode);
    }

    fn decode<B: Bus>(&mut self, bus: &mut B, opcode: u8) {
        match opcode {
            0x00 => self.exec_nop(),
            0x01 => self.exec_ld(bus, op::Reg16::BC, op::Imm16::with_offset(1), 3, 10),
            0x02 => self.exec_ld(bus, op::Ind8(op::Reg16::BC), op::Reg8::A, 1, 7),
            0x03 => self.exec_inc16(bus, op::Reg16::BC, 1, 6),
            0x04 => self.exec_inc8(bus, op::Reg8::B, 1, 4),
            0x05 => self.exec_dec8(bus, op::Reg8::B, 1, 4),
            0x06 => self.exec_ld(bus, op::Reg8::B, op::Imm8::with_offset(1), 2, 7),
            0x07 => self.exec_rlca(),
            0x08 => self.exec_ex(bus, op::Reg16::AF, op::Reg16::AF_, 1, 4),
            0x09 => self.exec_add16(bus, op::Reg16::HL, op::Reg16::BC, 1, 11),            
            0x0A => self.exec_ld(bus, op::Reg8::A, op::Ind8(op::Reg16::BC), 1, 7),
            0x0B => self.exec_dec16(bus, op::Reg16::BC, 1, 6),
            0x0C => self.exec_inc8(bus, op::Reg8::C, 1, 4),
            0x0D => self.exec_dec8(bus, op::Reg8::C, 1, 4),
            0x0E => self.exec_ld(bus, op::Reg8::C, op::Imm8::with_offset(1), 2, 7),
            0x0F => self.exec_rrca(),

            0x10 => self.exec_djnz(bus),
            0x11 => self.exec_ld(bus, op::Reg16::DE, op::Imm16::with_offset(1), 3, 10),
            0x12 => self.exec_ld(bus, op::Ind8(op::Reg16::DE), op::Reg8::A, 1, 7),
            0x13 => self.exec_inc16(bus, op::Reg16::DE, 1, 6),
            0x14 => self.exec_inc8(bus, op::Reg8::D, 1, 4),
            0x15 => self.exec_dec8(bus, op::Reg8::D, 1, 4),
            0x16 => self.exec_ld(bus, op::Reg8::C, op::Imm8::with_offset(1), 2, 7),
            0x17 => self.exec_rla(),            
            0x18 => self.exec_jr(bus, flag::Any),
            0x19 => self.exec_add16(bus, op::Reg16::HL, op::Reg16::DE, 1, 11),
            0x1A => self.exec_ld(bus, op::Reg8::A, op::Ind8(op::Reg16::DE), 1, 7),
            0x1B => self.exec_dec16(bus, op::Reg16::DE, 1, 6),
            0x1C => self.exec_inc8(bus, op::Reg8::E, 1, 4),
            0x1D => self.exec_dec8(bus, op::Reg8::E, 1, 4),
            0x1E => self.exec_ld(bus, op::Reg8::E, op::Imm8::with_offset(1), 2, 7),
            0x1F => self.exec_rra(),

            0x20 => self.exec_jr(bus, !flag::Z),
            0x21 => self.exec_ld(bus, op::Reg16::HL, op::Imm16::with_offset(1), 3, 10),
            0x22 => self.exec_ld(bus, op::Ind16(op::Imm16::with_offset(1)), op::Reg16::HL, 3, 16),
            0x23 => self.exec_inc16(bus, op::Reg16::HL, 1, 6),
            0x24 => self.exec_inc8(bus, op::Reg8::H, 1, 4),
            0x25 => self.exec_dec8(bus, op::Reg8::H, 1, 4),
            0x26 => self.exec_ld(bus, op::Reg8::H, op::Imm8::with_offset(1), 2, 7),
            0x27 => self.exec_daa(),
            0x28 => self.exec_jr(bus, flag::Z),
            0x29 => self.exec_add16(bus, op::Reg16::HL, op::Reg16::HL, 1, 11),
            0x2A => self.exec_ld(bus, op::Reg16::HL, op::Ind16(op::Imm16::with_offset(1)), 3, 16),
            0x2B => self.exec_dec16(bus, op::Reg16::HL, 1, 6),
            0x2C => self.exec_inc8(bus, op::Reg8::L, 1, 4),
            0x2D => self.exec_dec8(bus, op::Reg8::L, 1, 4),
            0x2E => self.exec_ld(bus, op::Reg8::L, op::Imm8::with_offset(1), 2, 7),
            0x2F => self.exec_cpl(),

            0x30 => self.exec_jr(bus, !flag::C),
            0x31 => self.exec_ld(bus, op::Reg16::SP, op::Imm16::with_offset(1), 3, 10),
            0x32 => self.exec_ld(bus, op::Ind8(op::Imm16::with_offset(1)), op::Reg8::A, 3, 13),
            0x33 => self.exec_inc16(bus, op::Reg16::SP, 1, 6),
            0x34 => self.exec_inc8(bus, op::Ind8(op::Reg16::HL), 1, 11),
            0x35 => self.exec_dec8(bus, op::Ind8(op::Reg16::HL), 1, 11),
            0x36 => self.exec_ld(bus, op::Ind8(op::Reg16::HL), op::Imm8::with_offset(1), 2, 10),
            0x37 => self.exec_scf(),
            0x38 => self.exec_jr(bus, flag::C),
            0x39 => self.exec_add16(bus, op::Reg16::HL, op::Reg16::SP, 1, 11),
            0x3A => self.exec_ld(bus, op::Reg8::A, op::Ind8(op::Imm16::with_offset(1)), 3, 13),
            0x3B => self.exec_dec16(bus, op::Reg16::SP, 1, 6),
            0x3C => self.exec_inc8(bus, op::Reg8::A, 1, 4),
            0x3D => self.exec_dec8(bus, op::Reg8::A, 1, 4),
            0x3E => self.exec_ld(bus, op::Reg8::A, op::Imm8::with_offset(1), 2, 7),
            0x3F => self.exec_ccf(),

            0x40 => self.exec_ld(bus, op::Reg8::B, op::Reg8::B, 1, 4),
            0x41 => self.exec_ld(bus, op::Reg8::B, op::Reg8::C, 1, 4),
            0x42 => self.exec_ld(bus, op::Reg8::B, op::Reg8::D, 1, 4),
            0x43 => self.exec_ld(bus, op::Reg8::B, op::Reg8::E, 1, 4),
            0x44 => self.exec_ld(bus, op::Reg8::B, op::Reg8::H, 1, 4),
            0x45 => self.exec_ld(bus, op::Reg8::B, op::Reg8::L, 1, 4),
            0x46 => self.exec_ld(bus, op::Reg8::B, op::Ind8(op::Reg16::HL), 1, 7),
            0x47 => self.exec_ld(bus, op::Reg8::B, op::Reg8::A, 1, 4),
            0x48 => self.exec_ld(bus, op::Reg8::C, op::Reg8::B, 1, 4),
            0x49 => self.exec_ld(bus, op::Reg8::C, op::Reg8::C, 1, 4),
            0x4A => self.exec_ld(bus, op::Reg8::C, op::Reg8::D, 1, 4),
            0x4B => self.exec_ld(bus, op::Reg8::C, op::Reg8::E, 1, 4),
            0x4C => self.exec_ld(bus, op::Reg8::C, op::Reg8::H, 1, 4),
            0x4D => self.exec_ld(bus, op::Reg8::C, op::Reg8::L, 1, 4),
            0x4E => self.exec_ld(bus, op::Reg8::C, op::Ind8(op::Reg16::HL), 1, 7),
            0x4F => self.exec_ld(bus, op::Reg8::C, op::Reg8::A, 1, 4),

            0x50 => self.exec_ld(bus, op::Reg8::D, op::Reg8::B, 1, 4),
            0x51 => self.exec_ld(bus, op::Reg8::D, op::Reg8::C, 1, 4),
            0x52 => self.exec_ld(bus, op::Reg8::D, op::Reg8::D, 1, 4),
            0x53 => self.exec_ld(bus, op::Reg8::D, op::Reg8::E, 1, 4),
            0x54 => self.exec_ld(bus, op::Reg8::D, op::Reg8::H, 1, 4),
            0x55 => self.exec_ld(bus, op::Reg8::D, op::Reg8::L, 1, 4),
            0x56 => self.exec_ld(bus, op::Reg8::D, op::Ind8(op::Reg16::HL), 1, 7),
            0x57 => self.exec_ld(bus, op::Reg8::D, op::Reg8::A, 1, 4),
            0x58 => self.exec_ld(bus, op::Reg8::E, op::Reg8::B, 1, 4),
            0x59 => self.exec_ld(bus, op::Reg8::E, op::Reg8::C, 1, 4),
            0x5A => self.exec_ld(bus, op::Reg8::E, op::Reg8::D, 1, 4),
            0x5B => self.exec_ld(bus, op::Reg8::E, op::Reg8::E, 1, 4),
            0x5C => self.exec_ld(bus, op::Reg8::E, op::Reg8::H, 1, 4),
            0x5D => self.exec_ld(bus, op::Reg8::E, op::Reg8::L, 1, 4),
            0x5E => self.exec_ld(bus, op::Reg8::E, op::Ind8(op::Reg16::HL), 1, 7),
            0x5F => self.exec_ld(bus, op::Reg8::E, op::Reg8::A, 1, 4),

            0x60 => self.exec_ld(bus, op::Reg8::H, op::Reg8::B, 1, 4),
            0x61 => self.exec_ld(bus, op::Reg8::H, op::Reg8::C, 1, 4),
            0x62 => self.exec_ld(bus, op::Reg8::H, op::Reg8::D, 1, 4),
            0x63 => self.exec_ld(bus, op::Reg8::H, op::Reg8::E, 1, 4),
            0x64 => self.exec_ld(bus, op::Reg8::H, op::Reg8::H, 1, 4),
            0x65 => self.exec_ld(bus, op::Reg8::H, op::Reg8::L, 1, 4),
            0x66 => self.exec_ld(bus, op::Reg8::H, op::Ind8(op::Reg16::HL), 1, 7),
            0x67 => self.exec_ld(bus, op::Reg8::H, op::Reg8::A, 1, 4),
            0x68 => self.exec_ld(bus, op::Reg8::L, op::Reg8::B, 1, 4),
            0x69 => self.exec_ld(bus, op::Reg8::L, op::Reg8::C, 1, 4),
            0x6A => self.exec_ld(bus, op::Reg8::L, op::Reg8::D, 1, 4),
            0x6B => self.exec_ld(bus, op::Reg8::L, op::Reg8::E, 1, 4),
            0x6C => self.exec_ld(bus, op::Reg8::L, op::Reg8::H, 1, 4),
            0x6D => self.exec_ld(bus, op::Reg8::L, op::Reg8::L, 1, 4),
            0x6E => self.exec_ld(bus, op::Reg8::L, op::Ind8(op::Reg16::HL), 1, 7),
            0x6F => self.exec_ld(bus, op::Reg8::L, op::Reg8::A, 1, 4),

            0x70 => self.exec_ld(bus, op::Ind8(op::Reg16::HL), op::Reg8::B, 1, 7),
            0x71 => self.exec_ld(bus, op::Ind8(op::Reg16::HL), op::Reg8::C, 1, 7),
            0x72 => self.exec_ld(bus, op::Ind8(op::Reg16::HL), op::Reg8::D, 1, 7),
            0x73 => self.exec_ld(bus, op::Ind8(op::Reg16::HL), op::Reg8::E, 1, 7),
            0x74 => self.exec_ld(bus, op::Ind8(op::Reg16::HL), op::Reg8::H, 1, 7),
            0x75 => self.exec_ld(bus, op::Ind8(op::Reg16::HL), op::Reg8::L, 1, 7),
            0x76 => self.halt(),
            0x77 => self.exec_ld(bus, op::Ind8(op::Reg16::HL), op::Reg8::A, 1, 7),
            0x78 => self.exec_ld(bus, op::Reg8::A, op::Reg8::B, 1, 4),
            0x79 => self.exec_ld(bus, op::Reg8::A, op::Reg8::C, 1, 4),
            0x7A => self.exec_ld(bus, op::Reg8::A, op::Reg8::D, 1, 4),
            0x7B => self.exec_ld(bus, op::Reg8::A, op::Reg8::E, 1, 4),
            0x7C => self.exec_ld(bus, op::Reg8::A, op::Reg8::H, 1, 4),
            0x7D => self.exec_ld(bus, op::Reg8::A, op::Reg8::L, 1, 4),
            0x7E => self.exec_ld(bus, op::Reg8::A, op::Ind8(op::Reg16::HL), 1, 7),
            0x7F => self.exec_ld(bus, op::Reg8::A, op::Reg8::A, 1, 4),

            0x80 => self.exec_add8(bus, op::Reg8::A, op::Reg8::B, false, 1, 4),
            0x81 => self.exec_add8(bus, op::Reg8::A, op::Reg8::C, false, 1, 4),
            0x82 => self.exec_add8(bus, op::Reg8::A, op::Reg8::D, false, 1, 4),
            0x83 => self.exec_add8(bus, op::Reg8::A, op::Reg8::E, false, 1, 4),
            0x84 => self.exec_add8(bus, op::Reg8::A, op::Reg8::H, false, 1, 4),
            0x85 => self.exec_add8(bus, op::Reg8::A, op::Reg8::L, false, 1, 4),
            0x86 => self.exec_add8(bus, op::Reg8::A, op::Ind8(op::Reg16::HL), false, 1, 7),
            0x87 => self.exec_add8(bus, op::Reg8::A, op::Reg8::A, false, 1, 4),
            0x88 => self.exec_add8(bus, op::Reg8::A, op::Reg8::B, true, 1, 4),
            0x89 => self.exec_add8(bus, op::Reg8::A, op::Reg8::C, true, 1, 4),
            0x8A => self.exec_add8(bus, op::Reg8::A, op::Reg8::D, true, 1, 4),
            0x8B => self.exec_add8(bus, op::Reg8::A, op::Reg8::E, true, 1, 4),
            0x8C => self.exec_add8(bus, op::Reg8::A, op::Reg8::H, true, 1, 4),
            0x8D => self.exec_add8(bus, op::Reg8::A, op::Reg8::L, true, 1, 4),
            0x8E => self.exec_add8(bus, op::Reg8::A, op::Ind8(op::Reg16::HL), true, 1, 7),
            0x8F => self.exec_add8(bus, op::Reg8::A, op::Reg8::A, true, 1, 4),

            0x90 => self.exec_sub8(bus, op::Reg8::A, op::Reg8::B, false, 1, 4),
            0x91 => self.exec_sub8(bus, op::Reg8::A, op::Reg8::C, false, 1, 4),
            0x92 => self.exec_sub8(bus, op::Reg8::A, op::Reg8::D, false, 1, 4),
            0x93 => self.exec_sub8(bus, op::Reg8::A, op::Reg8::E, false, 1, 4),
            0x94 => self.exec_sub8(bus, op::Reg8::A, op::Reg8::H, false, 1, 4),
            0x95 => self.exec_sub8(bus, op::Reg8::A, op::Reg8::L, false, 1, 4),
            0x96 => self.exec_sub8(bus, op::Reg8::A, op::Ind8(op::Reg16::HL), false, 1, 7),
            0x97 => self.exec_sub8(bus, op::Reg8::A, op::Reg8::A, false, 1, 4),
            0x98 => self.exec_sub8(bus, op::Reg8::A, op::Reg8::B, true, 1, 4),
            0x99 => self.exec_sub8(bus, op::Reg8::A, op::Reg8::C, true, 1, 4),
            0x9A => self.exec_sub8(bus, op::Reg8::A, op::Reg8::D, true, 1, 4),
            0x9B => self.exec_sub8(bus, op::Reg8::A, op::Reg8::E, true, 1, 4),
            0x9C => self.exec_sub8(bus, op::Reg8::A, op::Reg8::H, true, 1, 4),
            0x9D => self.exec_sub8(bus, op::Reg8::A, op::Reg8::L, true, 1, 4),
            0x9E => self.exec_sub8(bus, op::Reg8::A, op::Ind8(op::Reg16::HL), true, 1, 7),
            0x9F => self.exec_sub8(bus, op::Reg8::A, op::Reg8::A, true, 1, 4),

            0xA0 => self.exec_and(bus, op::Reg8::A, op::Reg8::B, 1, 4),
            0xA1 => self.exec_and(bus, op::Reg8::A, op::Reg8::C, 1, 4),
            0xA2 => self.exec_and(bus, op::Reg8::A, op::Reg8::D, 1, 4),
            0xA3 => self.exec_and(bus, op::Reg8::A, op::Reg8::E, 1, 4),
            0xA4 => self.exec_and(bus, op::Reg8::A, op::Reg8::H, 1, 4),
            0xA5 => self.exec_and(bus, op::Reg8::A, op::Reg8::L, 1, 4),
            0xA6 => self.exec_and(bus, op::Reg8::A, op::Ind8(op::Reg16::HL), 1, 7),
            0xA7 => self.exec_and(bus, op::Reg8::A, op::Reg8::A,  1, 4),
            0xA8 => self.exec_xor(bus, op::Reg8::A, op::Reg8::B, 1, 4),
            0xA9 => self.exec_xor(bus, op::Reg8::A, op::Reg8::C, 1, 4),
            0xAA => self.exec_xor(bus, op::Reg8::A, op::Reg8::D, 1, 4),
            0xAB => self.exec_xor(bus, op::Reg8::A, op::Reg8::E, 1, 4),
            0xAC => self.exec_xor(bus, op::Reg8::A, op::Reg8::H, 1, 4),
            0xAD => self.exec_xor(bus, op::Reg8::A, op::Reg8::L, 1, 4),
            0xAE => self.exec_xor(bus, op::Reg8::A, op::Ind8(op::Reg16::HL), 1, 7),
            0xAF => self.exec_xor(bus, op::Reg8::A, op::Reg8::A, 1, 4),

            0xB0 => self.exec_or(bus, op::Reg8::A, op::Reg8::B, 1, 4),
            0xB1 => self.exec_or(bus, op::Reg8::A, op::Reg8::C, 1, 4),
            0xB2 => self.exec_or(bus, op::Reg8::A, op::Reg8::D, 1, 4),
            0xB3 => self.exec_or(bus, op::Reg8::A, op::Reg8::E, 1, 4),
            0xB4 => self.exec_or(bus, op::Reg8::A, op::Reg8::H, 1, 4),
            0xB5 => self.exec_or(bus, op::Reg8::A, op::Reg8::L, 1, 4),
            0xB6 => self.exec_or(bus, op::Reg8::A, op::Ind8(op::Reg16::HL), 1, 7),
            0xB7 => self.exec_or(bus, op::Reg8::A, op::Reg8::A,  1, 4),
            0xB8 => self.exec_cp(bus, op::Reg8::A, op::Reg8::B, 1, 4),
            0xB9 => self.exec_cp(bus, op::Reg8::A, op::Reg8::C, 1, 4),
            0xBA => self.exec_cp(bus, op::Reg8::A, op::Reg8::D, 1, 4),
            0xBB => self.exec_cp(bus, op::Reg8::A, op::Reg8::E, 1, 4),
            0xBC => self.exec_cp(bus, op::Reg8::A, op::Reg8::H, 1, 4),
            0xBD => self.exec_cp(bus, op::Reg8::A, op::Reg8::L, 1, 4),
            0xBE => self.exec_cp(bus, op::Reg8::A, op::Ind8(op::Reg16::HL), 1, 7),
            0xBF => self.exec_cp(bus, op::Reg8::A, op::Reg8::A,  1, 4),

            0xC0 => self.exec_ret(bus, !flag::Z),
            0xC1 => self.exec_pop(bus, op::Reg16::BC, 1, 10),
            0xC2 => self.exec_jp(bus, !flag::Z, op::Imm16::with_offset(1), 3, 10),
            0xC3 => self.exec_jp(bus, flag::Any, op::Imm16::with_offset(1), 3, 10),
            0xC4 => self.exec_call(bus, !flag::Z),
            0xC5 => self.exec_push(bus, op::Reg16::BC, 1, 11),
            0xC6 => self.exec_add8(bus, op::Reg8::A, op::Imm8::with_offset(1), false, 1, 4),
            0xC7 => self.exec_rst(bus, 0x00),
            0xC8 => self.exec_ret(bus, flag::Z),
            0xC9 => self.exec_ret(bus, flag::Any),
            0xCA => self.exec_jp(bus, flag::Z, op::Imm16::with_offset(1), 3, 10),
            0xCB => todo!(),
            0xCC => self.exec_call(bus, flag::Z),
            0xCD => self.exec_call(bus, flag::Any),
            0xCE => self.exec_add8(bus, op::Reg8::A, op::Imm8::with_offset(1), true, 1, 4),
            0xCF => self.exec_rst(bus, 0x08),

            0xD0 => self.exec_ret(bus, !flag::C),
            0xD1 => self.exec_pop(bus, op::Reg16::DE, 1, 10),
            0xD2 => self.exec_jp(bus, !flag::C, op::Imm16::with_offset(1), 3, 10),
            0xD3 => self.exec_out(bus, op::Imm8::with_offset(1), op::Reg8::A, 2, 11),
            0xD4 => self.exec_call(bus, !flag::C),
            0xD5 => self.exec_push(bus, op::Reg16::DE, 1, 11),
            0xD6 => self.exec_sub8(bus, op::Reg8::A, op::Imm8::with_offset(1), false, 1, 4),
            0xD7 => self.exec_rst(bus, 0x10),
            0xD8 => self.exec_ret(bus, flag::C),
            0xD9 => self.exec_exx(),
            0xDA => self.exec_jp(bus, flag::C, op::Imm16::with_offset(1), 3, 10),
            0xDB => self.exec_in(bus, op::Reg8::A, op::Imm8::with_offset(1), 2, 11),
            0xDC => self.exec_call(bus, flag::C),
            0xDD => todo!(),
            0xDE => self.exec_sub8(bus, op::Reg8::A, op::Imm8::with_offset(1), true, 1, 4),
            0xDF => self.exec_rst(bus, 0x18),

            0xE0 => self.exec_ret(bus, !flag::P),
            0xE1 => self.exec_pop(bus, op::Reg16::HL, 1, 10),
            0xE2 => self.exec_jp(bus, !flag::P, op::Imm16::with_offset(1), 3, 10),
            0xE3 => self.exec_ex(bus, op::Ind16(op::Reg16::SP), op::Reg16::HL, 1, 19),
            0xE4 => self.exec_call(bus, !flag::P),
            0xE5 => self.exec_push(bus, op::Reg16::HL, 1, 11),
            0xE6 => todo!(),
            0xE7 => self.exec_rst(bus, 0x20),
            0xE8 => self.exec_ret(bus, flag::P),
            0xE9 => self.exec_jp(bus, flag::Any, op::Reg16::HL, 1, 4),
            0xEA => self.exec_jp(bus, flag::P, op::Imm16::with_offset(1), 3, 10),
            0xEB => self.exec_ex(bus, op::Reg16::DE, op::Reg16::HL, 1, 4),
            0xEC => self.exec_call(bus, flag::P),
            0xED => todo!(),
            0xEE => todo!(),
            0xEF => self.exec_rst(bus, 0x28),
            
            0xF0 => self.exec_ret(bus, !flag::N),
            0xF1 => self.exec_pop(bus, op::Reg16::AF, 1, 10),
            0xF2 => self.exec_jp(bus, !flag::N, op::Imm16::with_offset(1), 3, 10),
            0xF3 => todo!(),
            0xF4 => self.exec_call(bus, !flag::N),
            0xF5 => self.exec_push(bus, op::Reg16::AF, 1, 11),
            0xF6 => todo!(),
            0xF7 => self.exec_rst(bus, 0x30),
            0xF8 => self.exec_ret(bus, flag::N),
            0xF9 => self.exec_ld(bus, op::Reg16::SP, op::Reg16::HL, 1, 6),
            0xFA => self.exec_jp(bus, flag::N, op::Imm16::with_offset(1), 3, 10),
            0xFB => todo!(),
            0xFC => self.exec_call(bus, flag::N),
            0xFD => todo!(),
            0xFE => todo!(),
            0xFF => self.exec_rst(bus, 0x38),
        }
    }

    fn exec_add8<B, D, S> (&mut self, bus: &mut B, dst: D, src: S, with_carry: bool, size: usize, cycles: usize) 
    where B: Bus, D: op::DestOp<u8>, S: op::SrcOp<u8> {
        let mut ctx = op::Context::from(bus, &mut self.regs);
        let a = dst.get(&ctx);
        let mut b = src.get(&ctx);
        if with_carry && ctx.regs.flag(flag::C) {
            b += 1;
        }
        let c = a + b;
        dst.set(&mut ctx, c);

        self.regs.update_flags(
            flag::intrinsic(c) & 
            flag::H.on(flag::carry_nibble(a, c)) &
            flag::V.on(flag::overflow(a, b, c)) &
            flag::C.on(flag::carry_byte(a, c)) - flag::N
        );

        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_add16<B, D, S> (&mut self, bus: &mut B, dst: D, src: S, size: usize, cycles: usize) 
    where B: Bus, D: op::DestOp<u16>, S: op::SrcOp<u16> {
        let mut ctx = op::Context::from(bus, &mut self.regs);
        let a = dst.get(&ctx);
        let b = src.get(&ctx);
        let c = a + b;
        dst.set(&mut ctx, c);

        let ch = (c >> 8) as u8;

        self.regs.update_flags(
            flag::intrinsic_undocumented(ch) &
            flag::H.on(flag::carry(a, c, 0x0FFF)) &
            flag::C.on(flag::carry_word(a, c)) - flag::N
        );

        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_and<B, D, S> (&mut self, bus: &mut B, dst: D, src: S, size: usize, cycles: usize) 
    where B: Bus, D: op::DestOp<u8>, S: op::SrcOp<u8> {
        let mut ctx = op::Context::from(bus, &mut self.regs);
        let a = dst.get(&ctx);
        let b = src.get(&ctx);
        let c = a & b;
        dst.set(&mut ctx, c);

        self.regs.update_flags(
            flag::intrinsic(c) &
            flag::P.on(flag::parity(c)) &
            flag::C.on(flag::carry_byte(a, c)) + flag::H - flag::N - flag::C
        );

        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_call<B, P>(&mut self, bus: &mut B, pred: P)
    where B: Bus, P: flag::Predicate {
        let f = self.regs.flags();
        if pred.eval(f) {
            let addr = bus.mem_read_word(self.regs.pc() + 1);
            self.stack_push(bus, self.regs.pc() + 3);
            self.regs.set_pc(addr);
            self.cycles += 17;
        } else {
            self.regs.inc_pc(3);
            self.cycles += 10;
        }
    }

    fn exec_ccf(&mut self) {
        let f = self.regs.flags();
        let flag_c = flag::C.eval(f);
        self.regs.update_flags(            
            flag::intrinsic_undocumented(self.regs.a()) &
            flag::C.on(!flag_c) &
            flag::H.on(flag_c) + flag::N
        );
        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn exec_cp<B, D, S> (&mut self, bus: &mut B, dst: D, src: S, size: usize, cycles: usize) 
    where B: Bus, D: op::DestOp<u8>, S: op::SrcOp<u8> {
        let ctx = op::Context::from(bus, &mut self.regs);
        let a = dst.get(&ctx);
        let b = src.get(&ctx);
        let c = a - b;

        self.regs.update_flags(
            flag::S.on(flag::signed(c)) &
            flag::Z.on(c == 0) &
            flag::intrinsic_undocumented(b) &
            flag::H.on(flag::borrow_nibble(a, c)) &
            flag::V.on(flag::underflow(a, b, c)) &
            flag::C.on(flag::borrow_byte(a, c)) + flag::N
        );

        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_cpl(&mut self) {
        let a = self.regs.a();
        let c = !a;
        self.regs.set_a(c);

        self.regs.update_flags(flag::intrinsic_undocumented(c) + flag::N + flag::H);

        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn exec_daa(&mut self) {
        let mut reg_a = self.regs.a();
        let reg_ah = reg_a >> 4;
        let reg_al = reg_a & 0x0F;
        let reg_f = self.regs.flags();

        let flag_n = flag::N.eval(reg_f);
        let flag_h = flag::H.eval(reg_f);
        let flag_c = flag::C.eval(reg_f);

        let mut has_halfcarry = false;
        let mut has_carry = false;
        if reg_al > 9 || flag_h {
            if flag_n { reg_a -= 0x06 } else { reg_a += 0x06 }
            has_halfcarry = true;
        }
        if reg_ah > 9 || flag_c {
            if flag_n { reg_a -= 0x60 } else { reg_a += 0x60 }
            has_carry = true
        }
        self.regs.set_a(reg_a);

        self.regs.update_flags(
            flag::intrinsic(reg_a) &
            flag::C.on(has_carry) &
            flag::P.on(flag::parity(reg_a)) &
            flag::H.on(has_halfcarry)
        );

        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn exec_dec8<B, D> (&mut self, bus: &mut B, dst: D, size: usize, cycles: usize) 
    where B: Bus, D: op::DestOp<u8> {
        let mut ctx = op::Context::from(bus, &mut self.regs);
        let a = dst.get(&ctx) + 1;
        let c = a - 1;
        dst.set(&mut ctx, c);

        self.regs.update_flags(
            flag::intrinsic(c) & flag::H.on(flag::borrow_nibble(a, c)) & flag::V.on(flag::underflow(a, 1, c)) + flag::N
        );

        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_dec16<B, D> (&mut self, bus: &mut B, dst: D, size: usize, cycles: usize) 
    where B: Bus, D: op::DestOp<u16> {
        let mut ctx = op::Context::from(bus, &mut self.regs);
        let val = dst.get(&ctx) - 1;
        dst.set(&mut ctx, val);
        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_djnz<B: Bus>(&mut self, bus: &mut B) {
        let a = self.regs.c();
        let c = a - 1;
        if c != 0 {
            let rel = bus.mem_read(self.regs.pc() + 1) as i8;
            self.regs.inc_pc_signed(rel);
            self.cycles += 13;
        } else {
            self.regs.inc_pc(2);
            self.cycles += 8;
        }
    }

    fn exec_ex<B, D1, D2> (&mut self, bus: &mut B, opa: D1, opb: D2, size: usize, cycles: usize) 
    where B: Bus, D1: op::DestOp<u16>, D2: op::DestOp<u16> {
        let mut ctx = op::Context::from(bus, &mut self.regs);
        let a = opa.get(&ctx);
        let b = opb.get(&ctx);

        opa.set(&mut ctx, b);
        opb.set(&mut ctx, a);

        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_exx(&mut self){
        self.regs.swap_bc();
        self.regs.swap_de();
        self.regs.swap_hl();

        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn halt(&mut self) {
        self.cycles += 4;
    }

    fn exec_in<B, D, S>(&mut self, bus: &mut B, dst: D, src: S, size: usize, cycles: usize) 
    where B: Bus, D: op::DestOp<u8>, S: op::SrcOp<u8> {
        let mut ctx = op::Context::from(bus, &mut self.regs);
        let port = src.get(&ctx);        
        let val = ctx.bus.io_read(port);
        dst.set(&mut ctx, val);

        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_inc8<B, D> (&mut self, bus: &mut B, dst: D, size: usize, cycles: usize) 
    where B: Bus, D: op::DestOp<u8> {
        let mut ctx = op::Context::from(bus, &mut self.regs);
        let a = dst.get(&ctx) + 1;
        let c = a + 1;
        dst.set(&mut ctx, c);

        self.regs.update_flags(
            flag::intrinsic(c) & flag::H.on(flag::carry_nibble(a, c)) & flag::V.on(flag::overflow(a, 1, c)) - flag::N
        );

        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_inc16<B, D> (&mut self, bus: &mut B, dst: D, size: usize, cycles: usize) 
    where B: Bus, D: op::DestOp<u16> {
        let mut ctx = op::Context::from(bus, &mut self.regs);
        let val = dst.get(&ctx) + 1;
        dst.set(&mut ctx, val);
        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_jp<B, P, S>(&mut self, bus: &mut B, pred: P, dst: S, size: usize, cycles: usize) 
    where B: Bus, P: flag::Predicate, S: op::SrcOp<u16> {
        let f = self.regs.flags();
        if pred.eval(f) {
            let ctx = op::Context::from(bus, &mut self.regs);
            let addr = dst.get(&ctx);
            self.regs.set_pc(addr);
        } else {
            self.regs.inc_pc(size);
        }
        self.cycles += cycles;
    }

    fn exec_jr<B: Bus, P: flag::Predicate>(&mut self, bus: &mut B, pred: P) {
        let f = self.regs.flags();
        if pred.eval(f) {
            let rel = bus.mem_read(self.regs.pc() + 1) as i8;
            self.regs.inc_pc_signed(rel);
            self.cycles += 13;
        } else {
            self.regs.inc_pc(2);
            self.cycles += 7;
        }
    }

    fn exec_ld<T, B, D, S> (&mut self, bus: &mut B, dst: D, src: S, size: usize, cycles: usize) 
    where B: Bus, D: op::DestOp<T>, S: op::SrcOp<T> {
        let mut ctx = op::Context::from(bus, &mut self.regs);
        let val = src.get(&ctx);
        dst.set(&mut ctx, val);
        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_nop(&mut self) {
        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn exec_or<B, D, S> (&mut self, bus: &mut B, dst: D, src: S, size: usize, cycles: usize) 
    where B: Bus, D: op::DestOp<u8>, S: op::SrcOp<u8> {
        let mut ctx = op::Context::from(bus, &mut self.regs);
        let a = dst.get(&ctx);
        let b = src.get(&ctx);
        let c = a | b;
        dst.set(&mut ctx, c);

        self.regs.update_flags(
            flag::intrinsic(c) &
            flag::P.on(flag::parity(c)) &
            flag::C.on(flag::carry_byte(a, c)) - flag::H - flag::N - flag::C
        );

        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_out<B, S1, S2>(&mut self, bus: &mut B, dst: S1, src: S2, size: usize, cycles: usize) 
    where B: Bus, S1: op::SrcOp<u8>, S2: op::SrcOp<u8> {
        let ctx = op::Context::from(bus, &mut self.regs);
        let val = src.get(&ctx);
        let port = dst.get(&ctx);
        bus.io_write(port, val);

        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_pop<B, D>(&mut self, bus: &mut B, dst: D, size: usize, cycles: usize) 
    where B: Bus, D: op::DestOp<u16> {
        let val = self.stack_pop(bus);
        let mut ctx = op::Context::from(bus, &mut self.regs);
        dst.set(&mut ctx, val);

        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_push<B, S>(&mut self, bus: &mut B, src: S, size: usize, cycles: usize) 
    where B: Bus, S: op::SrcOp<u16> {
        let ctx = op::Context::from(bus, &mut self.regs);
        let val = src.get(&ctx);
        self.stack_push(bus, val);

        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_ret<B: Bus, P: flag::Predicate>(&mut self, bus: &mut B, pred: P) {
        if pred.eval(self.regs.flags()) {
            let addr = self.stack_pop(bus);
            self.regs.set_pc(addr);
            self.cycles += 11;
        } else {
            self.regs.inc_pc(1);
            self.cycles += 5;
        }
    }

    fn exec_rla(&mut self) {
        let a = self.regs.a();
        let mut c = a << 1;
        if self.regs.flag(flag::C) {
            c |= 0x01;
        }

        self.regs.set_a(c);

        self.regs.update_flags(
            flag::intrinsic_undocumented(c) & flag::C.on(a & 0x80 > 0) - flag::H - flag::N
        );

        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn exec_rlca(&mut self) {
        let a = self.regs.a();
        let c = (a << 1) | (a >> 7);

        self.regs.set_a(c);

        self.regs.update_flags(
            flag::intrinsic_undocumented(c) & flag::C.on(a & 0x80 > 0) - flag::H - flag::N
        );

        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn exec_rra(&mut self) {
        let a = self.regs.a();
        let mut c = a >> 1;
        if self.regs.flag(flag::C) {
            c |= 0x80;
        }

        self.regs.set_a(c);

        self.regs.update_flags(
            flag::intrinsic_undocumented(c) & flag::C.on(a & 0x01 > 0) - flag::H - flag::N
        );

        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn exec_rrca(&mut self) {
        let a = self.regs.a();
        let c = (a >> 1) | (a << 7);

        self.regs.set_a(c);

        self.regs.update_flags(
            flag::intrinsic_undocumented(c) & flag::C.on(a & 0x01 > 0) - flag::H - flag::N
        );

        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn exec_rst<B: Bus>(&mut self, bus: &mut B, addr: u16) {
        self.stack_push(bus, self.regs.pc() + 1);
        self.regs.set_pc(addr);
        self.cycles += 11;
    }

    fn exec_scf(&mut self) {
        self.regs.update_flags(
            flag::intrinsic_undocumented(self.regs.a()) + flag::C - flag::N - flag::H
        );
        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn exec_sub8<B, D, S> (&mut self, bus: &mut B, dst: D, src: S, with_carry: bool, size: usize, cycles: usize) 
    where B: Bus, D: op::DestOp<u8>, S: op::SrcOp<u8> {
        let mut ctx = op::Context::from(bus, &mut self.regs);
        let a = dst.get(&ctx);
        let mut b = src.get(&ctx);
        if with_carry && ctx.regs.flag(flag::C) {
            b += 1;
        }
        let c = a - b;
        dst.set(&mut ctx, c);

        self.regs.update_flags(
            flag::intrinsic(c) &
            flag::H.on(flag::borrow_nibble(a, c)) &
            flag::V.on(flag::underflow(a, b, c)) & 
            flag::C.on(flag::borrow_byte(a, c)) + flag::N
        );

        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_xor<B, D, S> (&mut self, bus: &mut B, dst: D, src: S, size: usize, cycles: usize) 
    where B: Bus, D: op::DestOp<u8>, S: op::SrcOp<u8> {
        let mut ctx = op::Context::from(bus, &mut self.regs);
        let a = dst.get(&ctx);
        let b = src.get(&ctx);
        let c = a ^ b;
        dst.set(&mut ctx, c);

        self.regs.update_flags(
            flag::intrinsic(c) &
            flag::P.on(flag::parity(c)) &
            flag::C.on(flag::carry_byte(a, c)) - flag::H - flag::N - flag::C
        );

        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn stack_pop<B: Bus>(&mut self, bus: &B) -> u16 {
        let val = bus.mem_read_word(self.regs.sp());
        self.regs.inc_sp(2);
        val
    }

    fn stack_push<B: Bus>(&mut self, bus: &mut B, val: u16) {
        self.regs.dec_sp(2);
        bus.mem_write_word(self.regs.sp(), val);
    }
}
