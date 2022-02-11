use crate::cpu::z80::bus::Bus;
use crate::cpu::z80::reg::Registers;
use crate::cpu::z80::flag::{self, Predicate};
use crate::cpu::z80::op::*;

pub struct CPU {
    regs: Registers,
    cycles: usize,

    iff1: bool,
    iff2: bool,
    
    flags_inc8: flag::PrecomputedUnary,
    flags_dec8: flag::PrecomputedUnary,
    flags_rla: flag::PrecomputedUnary,
    flags_rra: flag::PrecomputedUnary,
    flags_add8: flag::PrecomputedBinary,
    flags_sub8: flag::PrecomputedBinary,
    flags_and8: flag::PrecomputedBinary,
    flags_xor8: flag::PrecomputedBinary,
    flags_or8: flag::PrecomputedBinary,
}

impl CPU {
    pub fn new() -> Self {
        let mut cpu = Self {
            regs: Registers::new(),
            cycles: 0,
            iff1: false,
            iff2: false,
            flags_inc8: flag::PrecomputedUnary::for_inc8(),
            flags_dec8: flag::PrecomputedUnary::for_dec8(),
            flags_rla: flag::PrecomputedUnary::for_rla(),
            flags_rra: flag::PrecomputedUnary::for_rra(),
            flags_add8: flag::PrecomputedBinary::for_add8(),
            flags_sub8: flag::PrecomputedBinary::for_sub8(),
            flags_and8: flag::PrecomputedBinary::for_and8(),
            flags_xor8: flag::PrecomputedBinary::for_xor8(),
            flags_or8: flag::PrecomputedBinary::for_or8(),
        };
        cpu.reset();
        cpu
    }

    pub fn reset(&mut self) {
        self.regs.set_pc(0x0000);
        self.regs.set_af(0xFFFF);
        self.regs.set_sp(0xFFFF);
        self.iff1 = false;
        self.iff2 = false;
    }

    pub fn cycles(&self) -> usize { self.cycles }

    pub fn reset_cycles(&mut self) { self.cycles = 0 }

    pub fn regs(&self) -> &Registers { &self.regs }

    pub fn exec<B: Bus>(&mut self, b: &mut B) {
        let opcode = b.mem_read(self.regs.pc());
        self.decode(b, opcode);
    }

    fn decode(&mut self, bus: &mut impl Bus, opcode: u8) {
        match opcode {
            0x00 => self.exec_nop(1, 4),
            0x01 => self.exec_ld(bus, Reg16::BC, Imm16::with_offset(1), 3, 10),
            0x02 => self.exec_ld(bus, Ind8(Reg16::BC), Reg8::A, 1, 7),
            0x03 => self.exec_inc16(bus, Reg16::BC, 1, 6),
            0x04 => self.exec_inc8(bus, Reg8::B, 1, 4),
            0x05 => self.exec_dec8(bus, Reg8::B, 1, 4),
            0x06 => self.exec_ld(bus, Reg8::B, Imm8::with_offset(1), 2, 7),
            0x07 => self.exec_rlca(),
            0x08 => self.exec_ex(bus, Reg16::AF, Reg16::AF_, 1, 4),
            0x09 => self.exec_add16(bus, Reg16::HL, Reg16::BC, 1, 11),            
            0x0A => self.exec_ld(bus, Reg8::A, Ind8(Reg16::BC), 1, 7),
            0x0B => self.exec_dec16(bus, Reg16::BC, 1, 6),
            0x0C => self.exec_inc8(bus, Reg8::C, 1, 4),
            0x0D => self.exec_dec8(bus, Reg8::C, 1, 4),
            0x0E => self.exec_ld(bus, Reg8::C, Imm8::with_offset(1), 2, 7),
            0x0F => self.exec_rrca(),

            0x10 => self.exec_djnz(bus),
            0x11 => self.exec_ld(bus, Reg16::DE, Imm16::with_offset(1), 3, 10),
            0x12 => self.exec_ld(bus, Ind8(Reg16::DE), Reg8::A, 1, 7),
            0x13 => self.exec_inc16(bus, Reg16::DE, 1, 6),
            0x14 => self.exec_inc8(bus, Reg8::D, 1, 4),
            0x15 => self.exec_dec8(bus, Reg8::D, 1, 4),
            0x16 => self.exec_ld(bus, Reg8::D, Imm8::with_offset(1), 2, 7),
            0x17 => self.exec_rla(),            
            0x18 => self.exec_jr(bus, flag::Any),
            0x19 => self.exec_add16(bus, Reg16::HL, Reg16::DE, 1, 11),
            0x1A => self.exec_ld(bus, Reg8::A, Ind8(Reg16::DE), 1, 7),
            0x1B => self.exec_dec16(bus, Reg16::DE, 1, 6),
            0x1C => self.exec_inc8(bus, Reg8::E, 1, 4),
            0x1D => self.exec_dec8(bus, Reg8::E, 1, 4),
            0x1E => self.exec_ld(bus, Reg8::E, Imm8::with_offset(1), 2, 7),
            0x1F => self.exec_rra(),

            0x20 => self.exec_jr(bus, !flag::Z),
            0x21 => self.exec_ld(bus, Reg16::HL, Imm16::with_offset(1), 3, 10),
            0x22 => self.exec_ld(bus, Ind16(Imm16::with_offset(1)), Reg16::HL, 3, 16),
            0x23 => self.exec_inc16(bus, Reg16::HL, 1, 6),
            0x24 => self.exec_inc8(bus, Reg8::H, 1, 4),
            0x25 => self.exec_dec8(bus, Reg8::H, 1, 4),
            0x26 => self.exec_ld(bus, Reg8::H, Imm8::with_offset(1), 2, 7),
            0x27 => self.exec_daa(),
            0x28 => self.exec_jr(bus, flag::Z),
            0x29 => self.exec_add16(bus, Reg16::HL, Reg16::HL, 1, 11),
            0x2A => self.exec_ld(bus, Reg16::HL, Ind16(Imm16::with_offset(1)), 3, 16),
            0x2B => self.exec_dec16(bus, Reg16::HL, 1, 6),
            0x2C => self.exec_inc8(bus, Reg8::L, 1, 4),
            0x2D => self.exec_dec8(bus, Reg8::L, 1, 4),
            0x2E => self.exec_ld(bus, Reg8::L, Imm8::with_offset(1), 2, 7),
            0x2F => self.exec_cpl(),

            0x30 => self.exec_jr(bus, !flag::C),
            0x31 => self.exec_ld(bus, Reg16::SP, Imm16::with_offset(1), 3, 10),
            0x32 => self.exec_ld(bus, Ind8(Imm16::with_offset(1)), Reg8::A, 3, 13),
            0x33 => self.exec_inc16(bus, Reg16::SP, 1, 6),
            0x34 => self.exec_inc8(bus, Ind8(Reg16::HL), 1, 11),
            0x35 => self.exec_dec8(bus, Ind8(Reg16::HL), 1, 11),
            0x36 => self.exec_ld(bus, Ind8(Reg16::HL), Imm8::with_offset(1), 2, 10),
            0x37 => self.exec_scf(),
            0x38 => self.exec_jr(bus, flag::C),
            0x39 => self.exec_add16(bus, Reg16::HL, Reg16::SP, 1, 11),
            0x3A => self.exec_ld(bus, Reg8::A, Ind8(Imm16::with_offset(1)), 3, 13),
            0x3B => self.exec_dec16(bus, Reg16::SP, 1, 6),
            0x3C => self.exec_inc8(bus, Reg8::A, 1, 4),
            0x3D => self.exec_dec8(bus, Reg8::A, 1, 4),
            0x3E => self.exec_ld(bus, Reg8::A, Imm8::with_offset(1), 2, 7),
            0x3F => self.exec_ccf(),

            0x40 => self.exec_ld(bus, Reg8::B, Reg8::B, 1, 4),
            0x41 => self.exec_ld(bus, Reg8::B, Reg8::C, 1, 4),
            0x42 => self.exec_ld(bus, Reg8::B, Reg8::D, 1, 4),
            0x43 => self.exec_ld(bus, Reg8::B, Reg8::E, 1, 4),
            0x44 => self.exec_ld(bus, Reg8::B, Reg8::H, 1, 4),
            0x45 => self.exec_ld(bus, Reg8::B, Reg8::L, 1, 4),
            0x46 => self.exec_ld(bus, Reg8::B, Ind8(Reg16::HL), 1, 7),
            0x47 => self.exec_ld(bus, Reg8::B, Reg8::A, 1, 4),
            0x48 => self.exec_ld(bus, Reg8::C, Reg8::B, 1, 4),
            0x49 => self.exec_ld(bus, Reg8::C, Reg8::C, 1, 4),
            0x4A => self.exec_ld(bus, Reg8::C, Reg8::D, 1, 4),
            0x4B => self.exec_ld(bus, Reg8::C, Reg8::E, 1, 4),
            0x4C => self.exec_ld(bus, Reg8::C, Reg8::H, 1, 4),
            0x4D => self.exec_ld(bus, Reg8::C, Reg8::L, 1, 4),
            0x4E => self.exec_ld(bus, Reg8::C, Ind8(Reg16::HL), 1, 7),
            0x4F => self.exec_ld(bus, Reg8::C, Reg8::A, 1, 4),

            0x50 => self.exec_ld(bus, Reg8::D, Reg8::B, 1, 4),
            0x51 => self.exec_ld(bus, Reg8::D, Reg8::C, 1, 4),
            0x52 => self.exec_ld(bus, Reg8::D, Reg8::D, 1, 4),
            0x53 => self.exec_ld(bus, Reg8::D, Reg8::E, 1, 4),
            0x54 => self.exec_ld(bus, Reg8::D, Reg8::H, 1, 4),
            0x55 => self.exec_ld(bus, Reg8::D, Reg8::L, 1, 4),
            0x56 => self.exec_ld(bus, Reg8::D, Ind8(Reg16::HL), 1, 7),
            0x57 => self.exec_ld(bus, Reg8::D, Reg8::A, 1, 4),
            0x58 => self.exec_ld(bus, Reg8::E, Reg8::B, 1, 4),
            0x59 => self.exec_ld(bus, Reg8::E, Reg8::C, 1, 4),
            0x5A => self.exec_ld(bus, Reg8::E, Reg8::D, 1, 4),
            0x5B => self.exec_ld(bus, Reg8::E, Reg8::E, 1, 4),
            0x5C => self.exec_ld(bus, Reg8::E, Reg8::H, 1, 4),
            0x5D => self.exec_ld(bus, Reg8::E, Reg8::L, 1, 4),
            0x5E => self.exec_ld(bus, Reg8::E, Ind8(Reg16::HL), 1, 7),
            0x5F => self.exec_ld(bus, Reg8::E, Reg8::A, 1, 4),

            0x60 => self.exec_ld(bus, Reg8::H, Reg8::B, 1, 4),
            0x61 => self.exec_ld(bus, Reg8::H, Reg8::C, 1, 4),
            0x62 => self.exec_ld(bus, Reg8::H, Reg8::D, 1, 4),
            0x63 => self.exec_ld(bus, Reg8::H, Reg8::E, 1, 4),
            0x64 => self.exec_ld(bus, Reg8::H, Reg8::H, 1, 4),
            0x65 => self.exec_ld(bus, Reg8::H, Reg8::L, 1, 4),
            0x66 => self.exec_ld(bus, Reg8::H, Ind8(Reg16::HL), 1, 7),
            0x67 => self.exec_ld(bus, Reg8::H, Reg8::A, 1, 4),
            0x68 => self.exec_ld(bus, Reg8::L, Reg8::B, 1, 4),
            0x69 => self.exec_ld(bus, Reg8::L, Reg8::C, 1, 4),
            0x6A => self.exec_ld(bus, Reg8::L, Reg8::D, 1, 4),
            0x6B => self.exec_ld(bus, Reg8::L, Reg8::E, 1, 4),
            0x6C => self.exec_ld(bus, Reg8::L, Reg8::H, 1, 4),
            0x6D => self.exec_ld(bus, Reg8::L, Reg8::L, 1, 4),
            0x6E => self.exec_ld(bus, Reg8::L, Ind8(Reg16::HL), 1, 7),
            0x6F => self.exec_ld(bus, Reg8::L, Reg8::A, 1, 4),

            0x70 => self.exec_ld(bus, Ind8(Reg16::HL), Reg8::B, 1, 7),
            0x71 => self.exec_ld(bus, Ind8(Reg16::HL), Reg8::C, 1, 7),
            0x72 => self.exec_ld(bus, Ind8(Reg16::HL), Reg8::D, 1, 7),
            0x73 => self.exec_ld(bus, Ind8(Reg16::HL), Reg8::E, 1, 7),
            0x74 => self.exec_ld(bus, Ind8(Reg16::HL), Reg8::H, 1, 7),
            0x75 => self.exec_ld(bus, Ind8(Reg16::HL), Reg8::L, 1, 7),
            0x76 => self.halt(),
            0x77 => self.exec_ld(bus, Ind8(Reg16::HL), Reg8::A, 1, 7),
            0x78 => self.exec_ld(bus, Reg8::A, Reg8::B, 1, 4),
            0x79 => self.exec_ld(bus, Reg8::A, Reg8::C, 1, 4),
            0x7A => self.exec_ld(bus, Reg8::A, Reg8::D, 1, 4),
            0x7B => self.exec_ld(bus, Reg8::A, Reg8::E, 1, 4),
            0x7C => self.exec_ld(bus, Reg8::A, Reg8::H, 1, 4),
            0x7D => self.exec_ld(bus, Reg8::A, Reg8::L, 1, 4),
            0x7E => self.exec_ld(bus, Reg8::A, Ind8(Reg16::HL), 1, 7),
            0x7F => self.exec_ld(bus, Reg8::A, Reg8::A, 1, 4),

            0x80 => self.exec_add8(bus, Reg8::A, Reg8::B, false, 1, 4),
            0x81 => self.exec_add8(bus, Reg8::A, Reg8::C, false, 1, 4),
            0x82 => self.exec_add8(bus, Reg8::A, Reg8::D, false, 1, 4),
            0x83 => self.exec_add8(bus, Reg8::A, Reg8::E, false, 1, 4),
            0x84 => self.exec_add8(bus, Reg8::A, Reg8::H, false, 1, 4),
            0x85 => self.exec_add8(bus, Reg8::A, Reg8::L, false, 1, 4),
            0x86 => self.exec_add8(bus, Reg8::A, Ind8(Reg16::HL), false, 1, 7),
            0x87 => self.exec_add8(bus, Reg8::A, Reg8::A, false, 1, 4),
            0x88 => self.exec_add8(bus, Reg8::A, Reg8::B, true, 1, 4),
            0x89 => self.exec_add8(bus, Reg8::A, Reg8::C, true, 1, 4),
            0x8A => self.exec_add8(bus, Reg8::A, Reg8::D, true, 1, 4),
            0x8B => self.exec_add8(bus, Reg8::A, Reg8::E, true, 1, 4),
            0x8C => self.exec_add8(bus, Reg8::A, Reg8::H, true, 1, 4),
            0x8D => self.exec_add8(bus, Reg8::A, Reg8::L, true, 1, 4),
            0x8E => self.exec_add8(bus, Reg8::A, Ind8(Reg16::HL), true, 1, 7),
            0x8F => self.exec_add8(bus, Reg8::A, Reg8::A, true, 1, 4),

            0x90 => self.exec_sub8(bus, Reg8::A, Reg8::B, false, 1, 4),
            0x91 => self.exec_sub8(bus, Reg8::A, Reg8::C, false, 1, 4),
            0x92 => self.exec_sub8(bus, Reg8::A, Reg8::D, false, 1, 4),
            0x93 => self.exec_sub8(bus, Reg8::A, Reg8::E, false, 1, 4),
            0x94 => self.exec_sub8(bus, Reg8::A, Reg8::H, false, 1, 4),
            0x95 => self.exec_sub8(bus, Reg8::A, Reg8::L, false, 1, 4),
            0x96 => self.exec_sub8(bus, Reg8::A, Ind8(Reg16::HL), false, 1, 7),
            0x97 => self.exec_sub8(bus, Reg8::A, Reg8::A, false, 1, 4),
            0x98 => self.exec_sub8(bus, Reg8::A, Reg8::B, true, 1, 4),
            0x99 => self.exec_sub8(bus, Reg8::A, Reg8::C, true, 1, 4),
            0x9A => self.exec_sub8(bus, Reg8::A, Reg8::D, true, 1, 4),
            0x9B => self.exec_sub8(bus, Reg8::A, Reg8::E, true, 1, 4),
            0x9C => self.exec_sub8(bus, Reg8::A, Reg8::H, true, 1, 4),
            0x9D => self.exec_sub8(bus, Reg8::A, Reg8::L, true, 1, 4),
            0x9E => self.exec_sub8(bus, Reg8::A, Ind8(Reg16::HL), true, 1, 7),
            0x9F => self.exec_sub8(bus, Reg8::A, Reg8::A, true, 1, 4),

            0xA0 => self.exec_and(bus, Reg8::A, Reg8::B, 1, 4),
            0xA1 => self.exec_and(bus, Reg8::A, Reg8::C, 1, 4),
            0xA2 => self.exec_and(bus, Reg8::A, Reg8::D, 1, 4),
            0xA3 => self.exec_and(bus, Reg8::A, Reg8::E, 1, 4),
            0xA4 => self.exec_and(bus, Reg8::A, Reg8::H, 1, 4),
            0xA5 => self.exec_and(bus, Reg8::A, Reg8::L, 1, 4),
            0xA6 => self.exec_and(bus, Reg8::A, Ind8(Reg16::HL), 1, 7),
            0xA7 => self.exec_and(bus, Reg8::A, Reg8::A,  1, 4),
            0xA8 => self.exec_xor(bus, Reg8::A, Reg8::B, 1, 4),
            0xA9 => self.exec_xor(bus, Reg8::A, Reg8::C, 1, 4),
            0xAA => self.exec_xor(bus, Reg8::A, Reg8::D, 1, 4),
            0xAB => self.exec_xor(bus, Reg8::A, Reg8::E, 1, 4),
            0xAC => self.exec_xor(bus, Reg8::A, Reg8::H, 1, 4),
            0xAD => self.exec_xor(bus, Reg8::A, Reg8::L, 1, 4),
            0xAE => self.exec_xor(bus, Reg8::A, Ind8(Reg16::HL), 1, 7),
            0xAF => self.exec_xor(bus, Reg8::A, Reg8::A, 1, 4),

            0xB0 => self.exec_or(bus, Reg8::A, Reg8::B, 1, 4),
            0xB1 => self.exec_or(bus, Reg8::A, Reg8::C, 1, 4),
            0xB2 => self.exec_or(bus, Reg8::A, Reg8::D, 1, 4),
            0xB3 => self.exec_or(bus, Reg8::A, Reg8::E, 1, 4),
            0xB4 => self.exec_or(bus, Reg8::A, Reg8::H, 1, 4),
            0xB5 => self.exec_or(bus, Reg8::A, Reg8::L, 1, 4),
            0xB6 => self.exec_or(bus, Reg8::A, Ind8(Reg16::HL), 1, 7),
            0xB7 => self.exec_or(bus, Reg8::A, Reg8::A,  1, 4),
            0xB8 => self.exec_cp(bus, Reg8::A, Reg8::B, 1, 4),
            0xB9 => self.exec_cp(bus, Reg8::A, Reg8::C, 1, 4),
            0xBA => self.exec_cp(bus, Reg8::A, Reg8::D, 1, 4),
            0xBB => self.exec_cp(bus, Reg8::A, Reg8::E, 1, 4),
            0xBC => self.exec_cp(bus, Reg8::A, Reg8::H, 1, 4),
            0xBD => self.exec_cp(bus, Reg8::A, Reg8::L, 1, 4),
            0xBE => self.exec_cp(bus, Reg8::A, Ind8(Reg16::HL), 1, 7),
            0xBF => self.exec_cp(bus, Reg8::A, Reg8::A,  1, 4),

            0xC0 => self.exec_ret(bus, !flag::Z),
            0xC1 => self.exec_pop(bus, Reg16::BC, 1, 10),
            0xC2 => self.exec_jp(bus, !flag::Z, Imm16::with_offset(1), 3, 10),
            0xC3 => self.exec_jp(bus, flag::Any, Imm16::with_offset(1), 3, 10),
            0xC4 => self.exec_call(bus, !flag::Z),
            0xC5 => self.exec_push(bus, Reg16::BC, 1, 11),
            0xC6 => self.exec_add8(bus, Reg8::A, Imm8::with_offset(1), false, 2, 4),
            0xC7 => self.exec_rst(bus, 0x00),
            0xC8 => self.exec_ret(bus, flag::Z),
            0xC9 => self.exec_ret(bus, flag::Any),
            0xCA => self.exec_jp(bus, flag::Z, Imm16::with_offset(1), 3, 10),
            0xCB => todo!(),
            0xCC => self.exec_call(bus, flag::Z),
            0xCD => self.exec_call(bus, flag::Any),
            0xCE => self.exec_add8(bus, Reg8::A, Imm8::with_offset(1), true, 2, 4),
            0xCF => self.exec_rst(bus, 0x08),

            0xD0 => self.exec_ret(bus, !flag::C),
            0xD1 => self.exec_pop(bus, Reg16::DE, 1, 10),
            0xD2 => self.exec_jp(bus, !flag::C, Imm16::with_offset(1), 3, 10),
            0xD3 => self.exec_out(bus, Imm8::with_offset(1), Reg8::A, 2, 11),
            0xD4 => self.exec_call(bus, !flag::C),
            0xD5 => self.exec_push(bus, Reg16::DE, 1, 11),
            0xD6 => self.exec_sub8(bus, Reg8::A, Imm8::with_offset(1), false, 2, 4),
            0xD7 => self.exec_rst(bus, 0x10),
            0xD8 => self.exec_ret(bus, flag::C),
            0xD9 => self.exec_exx(),
            0xDA => self.exec_jp(bus, flag::C, Imm16::with_offset(1), 3, 10),
            0xDB => self.exec_in(bus, Some(Reg8::A), Imm8::with_offset(1), false, 2, 11),
            0xDC => self.exec_call(bus, flag::C),
            0xDD => todo!(),
            0xDE => self.exec_sub8(bus, Reg8::A, Imm8::with_offset(1), true, 2, 4),
            0xDF => self.exec_rst(bus, 0x18),

            0xE0 => self.exec_ret(bus, !flag::P),
            0xE1 => self.exec_pop(bus, Reg16::HL, 1, 10),
            0xE2 => self.exec_jp(bus, !flag::P, Imm16::with_offset(1), 3, 10),
            0xE3 => self.exec_ex(bus, Ind16(Reg16::SP), Reg16::HL, 1, 19),
            0xE4 => self.exec_call(bus, !flag::P),
            0xE5 => self.exec_push(bus, Reg16::HL, 1, 11),
            0xE6 => self.exec_and(bus, Reg8::A, Imm8::with_offset(1), 2, 7),
            0xE7 => self.exec_rst(bus, 0x20),
            0xE8 => self.exec_ret(bus, flag::P),
            0xE9 => self.exec_jp(bus, flag::Any, Reg16::HL, 1, 4),
            0xEA => self.exec_jp(bus, flag::P, Imm16::with_offset(1), 3, 10),
            0xEB => self.exec_ex(bus, Reg16::DE, Reg16::HL, 1, 4),
            0xEC => self.exec_call(bus, flag::P),
            0xED => self.decode_ext(bus, bus.mem_read(self.regs.pc() + 1)),
            0xEE => self.exec_xor(bus, Reg8::A, Imm8::with_offset(1), 2, 7),
            0xEF => self.exec_rst(bus, 0x28),
            
            0xF0 => self.exec_ret(bus, !flag::N),
            0xF1 => self.exec_pop(bus, Reg16::AF, 1, 10),
            0xF2 => self.exec_jp(bus, !flag::N, Imm16::with_offset(1), 3, 10),
            0xF3 => self.exec_di(),
            0xF4 => self.exec_call(bus, !flag::N),
            0xF5 => self.exec_push(bus, Reg16::AF, 1, 11),
            0xF6 => self.exec_or(bus, Reg8::A, Imm8::with_offset(1), 2, 7),
            0xF7 => self.exec_rst(bus, 0x30),
            0xF8 => self.exec_ret(bus, flag::N),
            0xF9 => self.exec_ld(bus, Reg16::SP, Reg16::HL, 1, 6),
            0xFA => self.exec_jp(bus, flag::N, Imm16::with_offset(1), 3, 10),
            0xFB => self.exec_ei(),
            0xFC => self.exec_call(bus, flag::N),
            0xFD => todo!(),
            0xFE => self.exec_cp(bus, Reg8::A, Imm8::with_offset(1), 2, 7),
            0xFF => self.exec_rst(bus, 0x38),
        }
    }

    fn decode_ext(&mut self, bus: &mut impl Bus, opcode: u8) {
        match opcode {
            0x40 => self.exec_in(bus, Some(Reg8::B), Reg8::C, true, 2, 12),
            0x41 => self.exec_out(bus, Reg8::C, Reg8::B, 2, 12),            
            0x48 => self.exec_in(bus, Some(Reg8::C), Reg8::C, true, 2, 12),
            0x49 => self.exec_out(bus, Reg8::C, Reg8::C, 2, 12),
            0x50 => self.exec_in(bus, Some(Reg8::D), Reg8::C, true, 2, 12),
            0x51 => self.exec_out(bus, Reg8::C, Reg8::D, 2, 12),
            0x58 => self.exec_in(bus, Some(Reg8::E), Reg8::C, true, 2, 12),
            0x59 => self.exec_out(bus, Reg8::C, Reg8::E, 2, 12),
            0x60 => self.exec_in(bus, Some(Reg8::H), Reg8::C, true, 2, 12),
            0x61 => self.exec_out(bus, Reg8::C, Reg8::H, 2, 12),
            0x68 => self.exec_in(bus, Some(Reg8::L), Reg8::C, true, 2, 12),
            0x69 => self.exec_out(bus, Reg8::C, Reg8::L, 2, 12),
            0x70 => self.exec_in(bus, None, Reg8::C, true, 2, 12),
            0x71 => self.exec_out(bus, Reg8::C, Lit8(0), 2, 12),
            0x78 => self.exec_in(bus, Some(Reg8::A), Reg8::C, true, 2, 12),
            0x79 => self.exec_out(bus, Reg8::C, Reg8::A, 2, 12),
            0xA0 => self.exec_ldi(bus, false),
            0xB0 => self.exec_ldi(bus, true),    
            _ => self.exec_nop(2, 8),
        }
    }

    fn exec_add8(&mut self, bus: &mut impl Bus, dst: impl DestOp<u8>, src: impl SrcOp<u8>, with_carry: bool, size: usize, cycles: usize) {
        let mut ctx = Context::from(bus, &mut self.regs);
        let a = dst.get(&ctx);
        let mut b = src.get(&ctx);
        if with_carry && ctx.regs.flag(flag::C) {
            b += 1;
        }
        let c = a + b;
        dst.set(&mut ctx, c);

        self.regs.update_flags(self.flags_add8.for_ops(a, b));
        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_add16(&mut self, bus: &mut impl Bus, dst: impl DestOp<u16>, src: impl SrcOp<u16>, size: usize, cycles: usize) {
        let mut ctx = Context::from(bus, &mut self.regs);
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

    fn exec_and(&mut self, bus: &mut impl Bus, dst: impl DestOp<u8>, src: impl SrcOp<u8>, size: usize, cycles: usize) {
        let mut ctx = Context::from(bus, &mut self.regs);
        let a = dst.get(&ctx);
        let b = src.get(&ctx);
        let c = a & b;
        dst.set(&mut ctx, c);

        self.regs.update_flags(self.flags_and8.for_ops(a, b));
        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_call(&mut self, bus: &mut impl Bus, pred: impl flag::Predicate){
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

    fn exec_cp(&mut self, bus: &mut impl Bus, dst: impl DestOp<u8>, src: impl SrcOp<u8>, size: usize, cycles: usize) {
        let ctx = Context::from(bus, &mut self.regs);
        let a = dst.get(&ctx);
        let b = src.get(&ctx);

        self.regs.update_flags(self.flags_sub8.for_ops(a, b));
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

    fn exec_dec8(&mut self, bus: &mut impl Bus, dst: impl DestOp<u8>, size: usize, cycles: usize) {
        let mut ctx = Context::from(bus, &mut self.regs);
        let a = dst.get(&ctx);
        let c = a - 1;
        dst.set(&mut ctx, c);

        self.regs.update_flags(self.flags_dec8.for_op(a));
        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_dec16(&mut self, bus: &mut impl Bus, dst: impl DestOp<u16>, size: usize, cycles: usize) {
        let mut ctx = Context::from(bus, &mut self.regs);
        let val = dst.get(&ctx) - 1;
        dst.set(&mut ctx, val);
        self.regs.inc_pc(size);
        self.cycles += cycles;
    }


    fn exec_di(&mut self) {
        self.iff1 = false;
        self.iff2 = false;
        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn exec_djnz(&mut self, bus: &mut impl Bus) {
        let a = self.regs.b();
        let c = a.wrapping_sub(1);
        self.regs.set_b(c);
        if c != 0 {
            let rel = bus.mem_read(self.regs.pc() + 1) as i8;
            self.regs.inc_pc_signed(rel.wrapping_add(2)); // as it is relative to the next instruction
            self.cycles += 13;
        } else {
            self.regs.inc_pc(2);
            self.cycles += 8;
        }
    }

    fn exec_ei(&mut self) {
        self.iff1 = true;
        self.iff2 = true;
        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn exec_ex(&mut self, bus: &mut impl Bus, opa: impl DestOp<u16>, opb: impl DestOp<u16>, size: usize, cycles: usize) {
        let mut ctx = Context::from(bus, &mut self.regs);
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

    fn exec_in(
        &mut self, 
        bus: &mut impl Bus, 
        dst: Option<Reg8>, 
        src: impl SrcOp<u8>, 
        flags_affected: bool, 
        size: usize, 
        cycles: usize,
    ) {
        let mut ctx = Context::from(bus, &mut self.regs);
        let port = src.get(&ctx);        
        let val = ctx.bus.io_read(port);
        if let Some(d) = dst {
            d.set(&mut ctx, val);
        }

        if flags_affected {
            self.regs.update_flags(flag::intrinsic(val) & flag::P.on(flag::parity(val)) - flag::H - flag::N);
        }

        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_inc8(&mut self, bus: &mut impl Bus, dst: impl DestOp<u8>, size: usize, cycles: usize) {
        let mut ctx = Context::from(bus, &mut self.regs);
        let a = dst.get(&ctx);
        let c = a + 1;
        dst.set(&mut ctx, c);

        self.regs.update_flags(self.flags_inc8.for_op(a));
        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_inc16(&mut self, bus: &mut impl Bus, dst: impl DestOp<u16>, size: usize, cycles: usize) {
        let mut ctx = Context::from(bus, &mut self.regs);
        let val = dst.get(&ctx) + 1;
        dst.set(&mut ctx, val);
        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_jp(&mut self, bus: &mut impl Bus, pred: impl flag::Predicate, dst: impl SrcOp<u16>, size: usize, cycles: usize) {
        let f = self.regs.flags();
        if pred.eval(f) {
            let ctx = Context::from(bus, &mut self.regs);
            let addr = dst.get(&ctx);
            self.regs.set_pc(addr);
        } else {
            self.regs.inc_pc(size);
        }
        self.cycles += cycles;
    }

    fn exec_jr(&mut self, bus: &mut impl Bus, pred: impl flag::Predicate) {
        let f = self.regs.flags();
        if pred.eval(f) {
            let rel = bus.mem_read(self.regs.pc().wrapping_add(1)) as i8;
            self.regs.inc_pc_signed(rel.wrapping_add(2)); // as it is relative to next instruction
            self.cycles += 13;
        } else {
            self.regs.inc_pc(2);
            self.cycles += 7;
        }
    }

    fn exec_ld<T>(&mut self, bus: &mut impl Bus, dst: impl DestOp<T>, src: impl SrcOp<T>, size: usize, cycles: usize) {
        let mut ctx = Context::from(bus, &mut self.regs);
        let val = src.get(&ctx);
        dst.set(&mut ctx, val);
        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_ldi(&mut self, bus: &mut impl Bus, repeat: bool) {
        let from = self.regs.hl();
        let to = self.regs.de();
        let byte = bus.mem_read(from);
        bus.mem_write(to, byte);
        let count = self.regs.bc();

        self.regs.set_hl(from.wrapping_add(1));
        self.regs.set_de(to.wrapping_add(1));
        self.regs.set_bc(count.wrapping_sub(1));

        let val_plus_a = byte.wrapping_add(self.regs.a());
        self.regs.update_flags(
            flag::PV.on(count > 0) &
            flag::F5.on(val_plus_a & 0x01 > 0) &
            flag::F3.on(val_plus_a & 0x08 > 0) - flag::H - flag::N,
        );

        if !repeat || self.regs.bc() == 0 {
            // LDI or LDIR with BC=0
            self.regs.inc_pc(2);
            self.cycles += 16;
        } else {
            self.cycles += 21;
        }
    }

    fn exec_nop(&mut self, size: usize, cycles: usize) {
        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_or(&mut self, bus: &mut impl Bus, dst: impl DestOp<u8>, src: impl SrcOp<u8>, size: usize, cycles: usize) {
        let mut ctx = Context::from(bus, &mut self.regs);
        let a = dst.get(&ctx);
        let b = src.get(&ctx);
        let c = a | b;
        dst.set(&mut ctx, c);

        self.regs.update_flags(self.flags_or8.for_ops(a, b));
        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_out(&mut self, bus: &mut impl Bus, dst: impl SrcOp<u8>, src: impl SrcOp<u8>, size: usize, cycles: usize) {
        let ctx = Context::from(bus, &mut self.regs);
        let val = src.get(&ctx);
        let port = dst.get(&ctx);
        bus.io_write(port, val);

        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_pop(&mut self, bus: &mut impl Bus, dst: impl DestOp<u16>, size: usize, cycles: usize) {
        let val = self.stack_pop(bus);
        let mut ctx = Context::from(bus, &mut self.regs);
        dst.set(&mut ctx, val);

        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_push(&mut self, bus: &mut impl Bus, src: impl SrcOp<u16>, size: usize, cycles: usize) {
        let ctx = Context::from(bus, &mut self.regs);
        let val = src.get(&ctx);
        self.stack_push(bus, val);

        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_ret(&mut self, bus: &mut impl Bus, pred: impl flag::Predicate) {
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

        self.regs.update_flags(self.flags_rla.for_op(a));
        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn exec_rlca(&mut self) {
        let a = self.regs.a();
        let c = (a << 1) | (a >> 7);

        self.regs.set_a(c);

        self.regs.update_flags(self.flags_rla.for_op(a));
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

        self.regs.update_flags(self.flags_rra.for_op(a));
        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn exec_rrca(&mut self) {
        let a = self.regs.a();
        let c = (a >> 1) | (a << 7);

        self.regs.set_a(c);

        self.regs.update_flags(self.flags_rra.for_op(a));
        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn exec_rst(&mut self, bus: &mut impl Bus, addr: u16) {
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

    fn exec_sub8(&mut self, bus: &mut impl Bus, dst: impl DestOp<u8>, src: impl SrcOp<u8>, with_carry: bool, size: usize, cycles: usize) {
        let mut ctx = Context::from(bus, &mut self.regs);
        let a = dst.get(&ctx);
        let mut b = src.get(&ctx);
        if with_carry && ctx.regs.flag(flag::C) {
            b += 1;
        }
        let c = a - b;
        dst.set(&mut ctx, c);

        self.regs.update_flags(self.flags_sub8.for_ops(a, b));
        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exec_xor(&mut self, bus: &mut impl Bus, dst: impl DestOp<u8>, src: impl SrcOp<u8>, size: usize, cycles: usize) {
        let mut ctx = Context::from(bus, &mut self.regs);
        let a = dst.get(&ctx);
        let b = src.get(&ctx);
        let c = a ^ b;
        dst.set(&mut ctx, c);

        self.regs.update_flags(self.flags_xor8.for_ops(a, b));
        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn stack_pop(&mut self, bus: &impl Bus) -> u16 {
        let val = bus.mem_read_word(self.regs.sp());
        self.regs.inc_sp(2);
        val
    }

    fn stack_push(&mut self, bus: &mut impl Bus, val: u16) {
        self.regs.dec_sp(2);
        bus.mem_write_word(self.regs.sp(), val);
    }
}

#[cfg(test)]
mod test {
    use rstest::*;
    
    use crate::cpu::z80::bus::FakeBus;

    use super::*;

    const FIXTURE_BC_ADDR: u16 = 0xF000;
    const FIXTURE_DE_ADDR: u16 = 0xF100;
    const FIXTURE_HL_ADDR: u16 = 0xF200;
    const FIXTURE_SP_ADDR: u16 = 0xF300;

    #[fixture]
    fn fixture() -> Fixture {
        let mut fix = Fixture {
            cpu: CPU::new(),
            bus: FakeBus::new(),
        };
        fix.cpu.regs.set_bc(FIXTURE_BC_ADDR);
        fix.cpu.regs.set_de(FIXTURE_DE_ADDR);
        fix.cpu.regs.set_hl(FIXTURE_HL_ADDR);
        fix.cpu.regs.set_sp(FIXTURE_SP_ADDR);
        fix
    }


    struct Fixture {
        cpu: CPU,
        bus: FakeBus,
    }

    impl Fixture {
        fn mem_write(&mut self, org: u16, data: &[u8]) {
            let mut i = org;
            for b in data {
                self.bus.mem_write(i, *b);
                i = i.wrapping_add(1);
            }
        }
    }

    trait OpGet<T> : FnOnce(&Fixture) -> T {}
    impl<T, F: FnOnce(&Fixture) -> T> OpGet<T> for F {}

    trait OpSet<T> : FnOnce(&mut Fixture, T) {}
    impl<T, F: FnOnce(&mut Fixture, T)> OpSet<T> for F {}

    // 8-bit getters
    fn get_a(f: &Fixture) -> u8 { f.cpu.regs.a() }
    fn get_b(f: &Fixture) -> u8 { f.cpu.regs.b() }
    fn get_c(f: &Fixture) -> u8 { f.cpu.regs.c() }
    fn get_d(f: &Fixture) -> u8 { f.cpu.regs.d() }
    fn get_e(f: &Fixture) -> u8 { f.cpu.regs.e() }
    fn get_h(f: &Fixture) -> u8 { f.cpu.regs.h() }
    fn get_l(f: &Fixture) -> u8 { f.cpu.regs.l() }
    fn get_ind_bc(f: &Fixture) ->u8 { f.bus.mem_read(f.cpu.regs.bc()) }
    fn get_ind_de(f: &Fixture) ->u8 { f.bus.mem_read(f.cpu.regs.de()) }    
    fn get_ind_hl(f: &Fixture) ->u8 { f.bus.mem_read(f.cpu.regs.hl()) }
    fn get_mem_byte(addr: u16) -> impl OpGet<u8> {
        move |f: &Fixture| f.bus.mem_read(addr)
    }

    // 16-bit getters
    fn get_bc(f: &Fixture) -> u16 { f.cpu.regs.bc() }
    fn get_de(f: &Fixture) -> u16 { f.cpu.regs.de() }
    fn get_hl(f: &Fixture) -> u16 { f.cpu.regs.hl() }
    fn get_sp(f: &Fixture) -> u16 { f.cpu.regs.sp() }
    fn get_mem_word(addr: u16) -> impl OpGet<u16> {
        move |f: &Fixture| f.bus.mem_read_word(addr)
    }

    // 8-bit setters
    fn set_no_byte(_: &mut Fixture, _: u8) {}
    fn set_a(f: &mut Fixture, v: u8) { f.cpu.regs.set_a(v) }
    fn set_b(f: &mut Fixture, v: u8) { f.cpu.regs.set_b(v) }
    fn set_c(f: &mut Fixture, v: u8) { f.cpu.regs.set_c(v) }
    fn set_d(f: &mut Fixture, v: u8) { f.cpu.regs.set_d(v) }
    fn set_e(f: &mut Fixture, v: u8) { f.cpu.regs.set_e(v) }
    fn set_h(f: &mut Fixture, v: u8) { f.cpu.regs.set_h(v) }
    fn set_l(f: &mut Fixture, v: u8) { f.cpu.regs.set_l(v) }
    fn set_ind_bc(f: &mut Fixture, v: u8) { f.bus.mem_write(FIXTURE_BC_ADDR, v) }
    fn set_ind_de(f: &mut Fixture, v: u8) { f.bus.mem_write(FIXTURE_DE_ADDR, v) }
    fn set_ind_hl(f: &mut Fixture, v: u8) { f.bus.mem_write(FIXTURE_HL_ADDR, v) }
    fn set_mem_byte(addr: u16) -> impl OpSet<u8> {
        move |f: &mut Fixture, v: u8| f.bus.mem_write(addr, v)
    }

    // 16-bit setters
    fn set_no_word(_: &mut Fixture, _: u16) {}
    fn set_hl(f: &mut Fixture, v: u16) { f.cpu.regs.set_hl(v) }
    fn set_mem_word(addr: u16) -> impl OpSet<u16> {
        move |f: &mut Fixture, v: u16| f.bus.mem_write_word(addr, v)
    }

    #[rstest]
    #[case(0x10, 0x1000, 10, 0x1012, 9)]
    #[case(-0x10, 0x1000, 10, 0x0FF2, 9)]
    #[case(0x10, 0x1000, 1, 0x1002, 0)]
    fn test_djnz(
        mut fixture: Fixture, 
        #[case] op: i8, 
        #[case] pc: u16,
        #[case] b: u8, 
        #[case] exp_pc: u16,
        #[case] exp_b: u8, 
    ) {
        fixture.mem_write(pc, &[0x10, op as u8]);
        fixture.cpu.regs.set_pc(pc);
        fixture.cpu.regs.set_b(b);

        fixture.cpu.exec(&mut fixture.bus);

        assert_eq!(fixture.cpu.regs.b(), exp_b);
        assert_eq!(fixture.cpu.regs.pc(), exp_pc);
    }

    #[rstest]
    #[case([0x18, 0xFA], 0x1000, 0b0000_0000, 0x0FFC)] // JR -6
    #[case([0x18, 0x06], 0x1000, 0b0000_0000, 0x1008)] // JR 6
    #[case([0x20, 0x06], 0x1000, 0b0000_0000, 0x1008)] // JR NZ, 6 ; jump
    #[case([0x20, 0x06], 0x1000, 0b0100_0000, 0x1002)] // JR NZ, 6 ; no jump
    #[case([0x28, 0x06], 0x1000, 0b0100_0000, 0x1008)] // JR Z, 6 ; jump
    #[case([0x28, 0x06], 0x1000, 0b0000_0000, 0x1002)] // JR Z, 6 ; no jump
    #[case([0x30, 0x06], 0x1000, 0b0000_0000, 0x1008)] // JR NC, 6 ; jump
    #[case([0x30, 0x06], 0x1000, 0b0000_0001, 0x1002)] // JR NC, 6 ; no jump
    #[case([0x38, 0x06], 0x1000, 0b0000_0001, 0x1008)] // JR C, 6 ; jump
    #[case([0x38, 0x06], 0x1000, 0b0000_0000, 0x1002)] // JR C, 6 ; no jump
    fn test_jr(
        mut fixture: Fixture, 
        #[case] opcode: [u8; 2], 
        #[case] pc: u16,
        #[case] f: u8, 
        #[case] exp_pc: u16,
    ) {
        fixture.mem_write(pc, &opcode);
        fixture.cpu.regs.set_pc(pc);
        fixture.cpu.regs.set_flags(f);

        fixture.cpu.exec(&mut fixture.bus);

        assert_eq!(fixture.cpu.regs.pc(), exp_pc);
    }

    #[rstest]
    /* 01: LD B,B    */ #[case(&[0x40], get_b, set_b, 0x0001)]      
    /* 02: LD B,C    */ #[case(&[0x41], get_b, set_c, 0x0001)]      
    /* 03: LD B,D    */ #[case(&[0x42], get_b, set_d, 0x0001)]      
    /* 04: LD B,E    */ #[case(&[0x43], get_b, set_e, 0x0001)]      
    /* 05: LD B,H    */ #[case(&[0x44], get_b, set_h, 0x0001)]      
    /* 06: LD B,L    */ #[case(&[0x45], get_b, set_l, 0x0001)]      
    /* 07: LD B,(HL) */ #[case(&[0x46], get_b, set_ind_hl, 0x0001)] 
    /* 08: LD B,A    */ #[case(&[0x47], get_b, set_a, 0x0001)]      
    /* 09: LD C,B    */ #[case(&[0x48], get_c, set_b, 0x0001)]      
    /* 10: LD C,C    */ #[case(&[0x49], get_c, set_c, 0x0001)]      
    /* 11: LD C,D    */ #[case(&[0x4A], get_c, set_d, 0x0001)]      
    /* 12: LD C,E    */ #[case(&[0x4B], get_c, set_e, 0x0001)]      
    /* 13: LD C,H    */ #[case(&[0x4C], get_c, set_h, 0x0001)]      
    /* 14: LD C,L    */ #[case(&[0x4D], get_c, set_l, 0x0001)]      
    /* 15: LD C,(HL) */ #[case(&[0x4E], get_c, set_ind_hl, 0x0001)] 
    /* 16: LD C,A    */ #[case(&[0x4F], get_c, set_a, 0x0001)]      
    /* 17: LD D,B    */ #[case(&[0x50], get_d, set_b, 0x0001)]      
    /* 18: LD D,C    */ #[case(&[0x51], get_d, set_c, 0x0001)]      
    /* 19: LD D,D    */ #[case(&[0x52], get_d, set_d, 0x0001)]      
    /* 20: LD D,E    */ #[case(&[0x53], get_d, set_e, 0x0001)]      
    /* 21: LD D,H    */ #[case(&[0x54], get_d, set_h, 0x0001)]      
    /* 22: LD D,L    */ #[case(&[0x55], get_d, set_l, 0x0001)]      
    /* 23: LD D,(HL) */ #[case(&[0x56], get_d, set_ind_hl, 0x0001)] 
    /* 24: LD D,A    */ #[case(&[0x57], get_d, set_a, 0x0001)]      
    /* 25: LD E,B    */ #[case(&[0x58], get_e, set_b, 0x0001)]      
    /* 26: LD E,C    */ #[case(&[0x59], get_e, set_c, 0x0001)]      
    /* 27: LD E,D    */ #[case(&[0x5A], get_e, set_d, 0x0001)]      
    /* 28: LD E,E    */ #[case(&[0x5B], get_e, set_e, 0x0001)]      
    /* 29: LD E,H    */ #[case(&[0x5C], get_e, set_h, 0x0001)]      
    /* 30: LD E,L    */ #[case(&[0x5D], get_e, set_l, 0x0001)]      
    /* 31: LD E,(HL) */ #[case(&[0x5E], get_e, set_ind_hl, 0x0001)] 
    /* 32: LD E,A    */ #[case(&[0x5F], get_e, set_a, 0x0001)]      
    /* 33: LD H,B    */ #[case(&[0x60], get_h, set_b, 0x0001)]      
    /* 34: LD H,C    */ #[case(&[0x61], get_h, set_c, 0x0001)]      
    /* 35: LD H,D    */ #[case(&[0x62], get_h, set_d, 0x0001)]      
    /* 36: LD H,E    */ #[case(&[0x63], get_h, set_e, 0x0001)]      
    /* 37: LD H,H    */ #[case(&[0x64], get_h, set_h, 0x0001)]      
    /* 38: LD H,L    */ #[case(&[0x65], get_h, set_l, 0x0001)]      
    /* 39: LD H,(HL) */ #[case(&[0x66], get_h, set_ind_hl, 0x0001)] 
    /* 40: LD H,A    */ #[case(&[0x67], get_h, set_a, 0x0001)]      
    /* 41: LD L,B    */ #[case(&[0x68], get_l, set_b, 0x0001)]      
    /* 42: LD L,C    */ #[case(&[0x69], get_l, set_c, 0x0001)]      
    /* 43: LD L,D    */ #[case(&[0x6A], get_l, set_d, 0x0001)]      
    /* 44: LD L,E    */ #[case(&[0x6B], get_l, set_e, 0x0001)]      
    /* 45: LD L,H    */ #[case(&[0x6C], get_l, set_h, 0x0001)]      
    /* 46: LD L,L    */ #[case(&[0x6D], get_l, set_l, 0x0001)]      
    /* 47: LD L,(HL) */ #[case(&[0x6E], get_l, set_ind_hl, 0x0001)] 
    /* 48: LD L,A    */ #[case(&[0x6F], get_l, set_a, 0x0001)]      
    /* 49: LD (HL),B */ #[case(&[0x70], get_ind_hl, set_b, 0x0001)]      
    /* 50: LD (HL),C */ #[case(&[0x71], get_ind_hl, set_c, 0x0001)]      
    /* 51: LD (HL),D */ #[case(&[0x72], get_ind_hl, set_d, 0x0001)]      
    /* 52: LD (HL),E */ #[case(&[0x73], get_ind_hl, set_e, 0x0001)]      
    /* 53: LD (HL),H */ #[case(&[0x74], get_ind_hl, set_h, 0x0001)]      
    /* 54: LD (HL),L */ #[case(&[0x75], get_ind_hl, set_l, 0x0001)]      
    /* 55: LD (HL),A */ #[case(&[0x77], get_ind_hl, set_a, 0x0001)]      
    /* 56: LD A,B    */ #[case(&[0x78], get_a, set_b, 0x0001)]      
    /* 57: LD A,C    */ #[case(&[0x79], get_a, set_c, 0x0001)]      
    /* 58: LD A,D    */ #[case(&[0x7A], get_a, set_d, 0x0001)]      
    /* 59: LD A,E    */ #[case(&[0x7B], get_a, set_e, 0x0001)]      
    /* 60: LD A,H    */ #[case(&[0x7C], get_a, set_h, 0x0001)]      
    /* 61: LD A,L    */ #[case(&[0x7D], get_a, set_l, 0x0001)]      
    /* 62: LD A,(HL) */ #[case(&[0x7E], get_a, set_ind_hl, 0x0001)] 
    /* 63: LD A,A    */ #[case(&[0x7F], get_a, set_a, 0x0001)]      
    /* 64: LD (BC),A */ #[case(&[0x02], get_ind_bc, set_a, 0x0001)]      
    /* 65: LD (DE),A */ #[case(&[0x12], get_ind_de, set_a, 0x0001)]      
    /* 66: LD (**),A */ #[case(&[0x32, 0x00, 0xD0], get_mem_byte(0xD000), set_a, 0x0003)]
    /* 67: LD B,*    */ #[case(&[0x06, 0x42], get_b, set_no_byte, 0x0002)]
    /* 68: LD D,*    */ #[case(&[0x16, 0x42], get_d, set_no_byte, 0x0002)]
    /* 69: LD H,*    */ #[case(&[0x26, 0x42], get_h, set_no_byte, 0x0002)]
    /* 70: LD (HL),* */ #[case(&[0x36, 0x42], get_ind_hl, set_no_byte, 0x0002)]    
    /* 71: LD A,(BC) */ #[case(&[0x0A], get_a, set_ind_bc, 0x0001)]
    /* 72: LD A,(DE) */ #[case(&[0x1A], get_a, set_ind_de, 0x0001)]
    /* 73: LD A,(**) */ #[case(&[0x3A, 0x00, 0xD0], get_a, set_mem_byte(0xD000), 0x0003)]
    /* 74: LD C,*    */ #[case(&[0x0E, 0x42], get_c, set_no_byte, 0x0002)]
    /* 75: LD E,*    */ #[case(&[0x1E, 0x42], get_e, set_no_byte, 0x0002)]
    /* 76: LD L,*    */ #[case(&[0x2E, 0x42], get_l, set_no_byte, 0x0002)]
    /* 77: LD A,*    */ #[case(&[0x3E, 0x42], get_a, set_no_byte, 0x0002)]
    fn test_ld8(
        mut fixture: Fixture,
        #[case] opcode: &[u8],
        #[case] get: impl OpGet<u8>,
        #[case] set: impl OpSet<u8>,
        #[case] expected_pc: u16,        
    ) {
        fixture.mem_write(0x0000, opcode);
        set(&mut fixture, 0x42);

        fixture.cpu.exec(&mut fixture.bus);

        assert_eq!(get(&fixture), 0x42);
        assert_eq!(fixture.cpu.regs.pc(), expected_pc);
    }

    #[rstest]
    /* 01: LD BC,**     */ #[case(&[0x01, 0xCD, 0xAB], get_bc, set_no_word, 0x0003)]      
    /* 02: LD DE,**     */ #[case(&[0x11, 0xCD, 0xAB], get_de, set_no_word, 0x0003)]      
    /* 03: LD HL,**     */ #[case(&[0x21, 0xCD, 0xAB], get_hl, set_no_word, 0x0003)]      
    /* 04: LD SP,**     */ #[case(&[0x31, 0xCD, 0xAB], get_sp, set_no_word, 0x0003)]      
    /* 05: LD (**),HL   */ #[case(&[0x22, 0x00, 0xD0], get_mem_word(0xD000), set_hl, 0x0003)]      
    /* 06: LD HL,(**)   */ #[case(&[0x2A, 0x00, 0xD0], get_hl, set_mem_word(0xD000), 0x0003)]      
    /* 07: LD SP,HL     */ #[case(&[0xF9], get_sp, set_hl, 0x0001)]      
    fn test_ld16(
        mut fixture: Fixture,
        #[case] opcode: &[u8],
        #[case] get: impl OpGet<u16>,
        #[case] set: impl OpSet<u16>,
        #[case] expected_pc: u16,        
    ) {
        fixture.mem_write(0x0000, opcode);
        set(&mut fixture, 0xABCD);

        fixture.cpu.exec(&mut fixture.bus);

        assert_eq!(get(&fixture), 0xABCD);
        assert_eq!(fixture.cpu.regs.pc(), expected_pc);
    }
}