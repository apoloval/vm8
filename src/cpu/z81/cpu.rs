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
            0x00 => self.nop(),
            0x01 => self.ld(bus, op::Reg16::BC, op::Imm16::with_offset(1), 3, 10),
            0x02 => self.ld(bus, op::Ind8(op::Reg16::BC), op::Reg8::A, 1, 7),
            0x03 => self.inc16(bus, op::Reg16::BC, 1, 6),
            0x04 => self.inc8(bus, op::Reg8::B, 1, 4),
            0x05 => self.dec8(bus, op::Reg8::B, 1, 4),
            0x06 => self.ld(bus, op::Reg8::B, op::Imm8::with_offset(1), 2, 7),
            0x07 => self.rlca(),
            0x08 => self.ex(bus, op::Reg16::AF, op::Reg16::AF_, 1, 4),
            0x09 => self.add16(bus, op::Reg16::HL, op::Reg16::BC, 1, 11),            
            0x0A => self.ld(bus, op::Reg8::A, op::Ind8(op::Reg16::BC), 1, 7),
            0x0B => self.dec16(bus, op::Reg16::BC, 1, 6),
            0x0C => self.inc8(bus, op::Reg8::C, 1, 4),
            0x0D => self.dec8(bus, op::Reg8::C, 1, 4),
            0x0E => self.ld(bus, op::Reg8::C, op::Imm8::with_offset(1), 2, 7),
            0x0F => self.rrca(),

            0x10 => self.djnz(bus),
            0x11 => self.ld(bus, op::Reg16::DE, op::Imm16::with_offset(1), 3, 10),
            0x12 => self.ld(bus, op::Ind8(op::Reg16::DE), op::Reg8::A, 1, 7),
            0x13 => self.inc16(bus, op::Reg16::DE, 1, 6),
            0x14 => self.inc8(bus, op::Reg8::D, 1, 4),
            0x15 => self.dec8(bus, op::Reg8::D, 1, 4),
            0x16 => self.ld(bus, op::Reg8::C, op::Imm8::with_offset(1), 2, 7),
            0x17 => self.rla(),            
            0x18 => self.jr(bus, flag::Any),
            0x19 => self.add16(bus, op::Reg16::HL, op::Reg16::DE, 1, 11),
            0x1A => self.ld(bus, op::Reg8::A, op::Ind8(op::Reg16::DE), 1, 7),
            0x1B => self.dec16(bus, op::Reg16::DE, 1, 6),
            0x1C => self.inc8(bus, op::Reg8::E, 1, 4),
            0x1D => self.dec8(bus, op::Reg8::E, 1, 4),
            0x1E => self.ld(bus, op::Reg8::E, op::Imm8::with_offset(1), 2, 7),
            0x1F => self.rra(),

            0x20 => self.jr(bus, !flag::Z),
            0x21 => self.ld(bus, op::Reg16::HL, op::Imm16::with_offset(1), 3, 10),
            0x22 => self.ld(bus, op::Ind16(op::Imm16::with_offset(1)), op::Reg16::HL, 3, 16),
            0x23 => self.inc16(bus, op::Reg16::HL, 1, 6),
            0x24 => self.inc8(bus, op::Reg8::H, 1, 4),
            0x25 => self.dec8(bus, op::Reg8::H, 1, 4),
            0x26 => self.ld(bus, op::Reg8::H, op::Imm8::with_offset(1), 2, 7),
            0x27 => self.daa(),
            0x28 => self.jr(bus, flag::Z),
            0x29 => self.add16(bus, op::Reg16::HL, op::Reg16::HL, 1, 11),
            0x2A => self.ld(bus, op::Reg16::HL, op::Ind16(op::Imm16::with_offset(1)), 3, 16),
            0x2B => self.dec16(bus, op::Reg16::HL, 1, 6),
            0x2C => self.inc8(bus, op::Reg8::L, 1, 4),
            0x2D => self.dec8(bus, op::Reg8::L, 1, 4),
            0x2E => self.ld(bus, op::Reg8::L, op::Imm8::with_offset(1), 2, 7),
            0x2F => self.cpl(),

            0x30 => self.jr(bus, !flag::C),
            0x31 => self.ld(bus, op::Reg16::SP, op::Imm16::with_offset(1), 3, 10),
            0x32 => self.ld(bus, op::Ind8(op::Imm16::with_offset(1)), op::Reg8::A, 3, 13),
            0x33 => self.inc16(bus, op::Reg16::SP, 1, 6),
            0x34 => self.inc8(bus, op::Ind8(op::Reg16::HL), 1, 11),
            0x35 => self.dec8(bus, op::Ind8(op::Reg16::HL), 1, 11),
            0x36 => self.ld(bus, op::Ind8(op::Reg16::HL), op::Imm8::with_offset(1), 2, 10),
            0x37 => self.scf(),
            0x38 => self.jr(bus, flag::C),
            0x39 => self.add16(bus, op::Reg16::HL, op::Reg16::SP, 1, 11),
            0x3A => self.ld(bus, op::Reg8::A, op::Ind8(op::Imm16::with_offset(1)), 3, 13),
            0x3B => self.dec16(bus, op::Reg16::SP, 1, 6),
            0x3C => self.inc8(bus, op::Reg8::A, 1, 4),
            0x3D => self.dec8(bus, op::Reg8::A, 1, 4),
            0x3E => self.ld(bus, op::Reg8::A, op::Imm8::with_offset(1), 2, 7),
            0x3F => self.ccf(),

            0x40 => self.ld(bus, op::Reg8::B, op::Reg8::B, 1, 4),
            0x41 => self.ld(bus, op::Reg8::B, op::Reg8::C, 1, 4),
            0x42 => self.ld(bus, op::Reg8::B, op::Reg8::D, 1, 4),
            0x43 => self.ld(bus, op::Reg8::B, op::Reg8::E, 1, 4),
            0x44 => self.ld(bus, op::Reg8::B, op::Reg8::H, 1, 4),
            0x45 => self.ld(bus, op::Reg8::B, op::Reg8::L, 1, 4),
            0x46 => self.ld(bus, op::Reg8::B, op::Ind8(op::Reg16::HL), 1, 7),
            0x47 => self.ld(bus, op::Reg8::B, op::Reg8::A, 1, 4),
            0x48 => self.ld(bus, op::Reg8::C, op::Reg8::B, 1, 4),
            0x49 => self.ld(bus, op::Reg8::C, op::Reg8::C, 1, 4),
            0x4A => self.ld(bus, op::Reg8::C, op::Reg8::D, 1, 4),
            0x4B => self.ld(bus, op::Reg8::C, op::Reg8::E, 1, 4),
            0x4C => self.ld(bus, op::Reg8::C, op::Reg8::H, 1, 4),
            0x4D => self.ld(bus, op::Reg8::C, op::Reg8::L, 1, 4),
            0x4E => self.ld(bus, op::Reg8::C, op::Ind8(op::Reg16::HL), 1, 7),
            0x4F => self.ld(bus, op::Reg8::C, op::Reg8::A, 1, 4),

            0x50 => self.ld(bus, op::Reg8::D, op::Reg8::B, 1, 4),
            0x51 => self.ld(bus, op::Reg8::D, op::Reg8::C, 1, 4),
            0x52 => self.ld(bus, op::Reg8::D, op::Reg8::D, 1, 4),
            0x53 => self.ld(bus, op::Reg8::D, op::Reg8::E, 1, 4),
            0x54 => self.ld(bus, op::Reg8::D, op::Reg8::H, 1, 4),
            0x55 => self.ld(bus, op::Reg8::D, op::Reg8::L, 1, 4),
            0x56 => self.ld(bus, op::Reg8::D, op::Ind8(op::Reg16::HL), 1, 7),
            0x57 => self.ld(bus, op::Reg8::D, op::Reg8::A, 1, 4),
            0x58 => self.ld(bus, op::Reg8::E, op::Reg8::B, 1, 4),
            0x59 => self.ld(bus, op::Reg8::E, op::Reg8::C, 1, 4),
            0x5A => self.ld(bus, op::Reg8::E, op::Reg8::D, 1, 4),
            0x5B => self.ld(bus, op::Reg8::E, op::Reg8::E, 1, 4),
            0x5C => self.ld(bus, op::Reg8::E, op::Reg8::H, 1, 4),
            0x5D => self.ld(bus, op::Reg8::E, op::Reg8::L, 1, 4),
            0x5E => self.ld(bus, op::Reg8::E, op::Ind8(op::Reg16::HL), 1, 7),
            0x5F => self.ld(bus, op::Reg8::E, op::Reg8::A, 1, 4),

            0x60 => self.ld(bus, op::Reg8::H, op::Reg8::B, 1, 4),
            0x61 => self.ld(bus, op::Reg8::H, op::Reg8::C, 1, 4),
            0x62 => self.ld(bus, op::Reg8::H, op::Reg8::D, 1, 4),
            0x63 => self.ld(bus, op::Reg8::H, op::Reg8::E, 1, 4),
            0x64 => self.ld(bus, op::Reg8::H, op::Reg8::H, 1, 4),
            0x65 => self.ld(bus, op::Reg8::H, op::Reg8::L, 1, 4),
            0x66 => self.ld(bus, op::Reg8::H, op::Ind8(op::Reg16::HL), 1, 7),
            0x67 => self.ld(bus, op::Reg8::H, op::Reg8::A, 1, 4),
            0x68 => self.ld(bus, op::Reg8::L, op::Reg8::B, 1, 4),
            0x69 => self.ld(bus, op::Reg8::L, op::Reg8::C, 1, 4),
            0x6A => self.ld(bus, op::Reg8::L, op::Reg8::D, 1, 4),
            0x6B => self.ld(bus, op::Reg8::L, op::Reg8::E, 1, 4),
            0x6C => self.ld(bus, op::Reg8::L, op::Reg8::H, 1, 4),
            0x6D => self.ld(bus, op::Reg8::L, op::Reg8::L, 1, 4),
            0x6E => self.ld(bus, op::Reg8::L, op::Ind8(op::Reg16::HL), 1, 7),
            0x6F => self.ld(bus, op::Reg8::L, op::Reg8::A, 1, 4),

            0x70 => self.ld(bus, op::Ind8(op::Reg16::HL), op::Reg8::B, 1, 7),
            0x71 => self.ld(bus, op::Ind8(op::Reg16::HL), op::Reg8::C, 1, 7),
            0x72 => self.ld(bus, op::Ind8(op::Reg16::HL), op::Reg8::D, 1, 7),
            0x73 => self.ld(bus, op::Ind8(op::Reg16::HL), op::Reg8::E, 1, 7),
            0x74 => self.ld(bus, op::Ind8(op::Reg16::HL), op::Reg8::H, 1, 7),
            0x75 => self.ld(bus, op::Ind8(op::Reg16::HL), op::Reg8::L, 1, 7),
            0x76 => todo!(),
            0x77 => self.ld(bus, op::Ind8(op::Reg16::HL), op::Reg8::A, 1, 7),
            0x78 => self.ld(bus, op::Reg8::A, op::Reg8::B, 1, 4),
            0x79 => self.ld(bus, op::Reg8::A, op::Reg8::C, 1, 4),
            0x7A => self.ld(bus, op::Reg8::A, op::Reg8::D, 1, 4),
            0x7B => self.ld(bus, op::Reg8::A, op::Reg8::E, 1, 4),
            0x7C => self.ld(bus, op::Reg8::A, op::Reg8::H, 1, 4),
            0x7D => self.ld(bus, op::Reg8::A, op::Reg8::L, 1, 4),
            0x7E => self.ld(bus, op::Reg8::A, op::Ind8(op::Reg16::HL), 1, 7),
            0x7F => self.ld(bus, op::Reg8::A, op::Reg8::A, 1, 4),

            0x80 => self.add8(bus, op::Reg8::A, op::Reg8::B, false, 1, 4),
            0x81 => self.add8(bus, op::Reg8::A, op::Reg8::C, false, 1, 4),
            0x82 => self.add8(bus, op::Reg8::A, op::Reg8::D, false, 1, 4),
            0x83 => self.add8(bus, op::Reg8::A, op::Reg8::E, false, 1, 4),
            0x84 => self.add8(bus, op::Reg8::A, op::Reg8::H, false, 1, 4),
            0x85 => self.add8(bus, op::Reg8::A, op::Reg8::L, false, 1, 4),
            0x86 => self.add8(bus, op::Reg8::A, op::Ind8(op::Reg16::HL), false, 1, 7),
            0x87 => self.add8(bus, op::Reg8::A, op::Reg8::A, false, 1, 4),
            0x88 => self.add8(bus, op::Reg8::A, op::Reg8::B, true, 1, 4),
            0x89 => self.add8(bus, op::Reg8::A, op::Reg8::C, true, 1, 4),
            0x8A => self.add8(bus, op::Reg8::A, op::Reg8::D, true, 1, 4),
            0x8B => self.add8(bus, op::Reg8::A, op::Reg8::E, true, 1, 4),
            0x8C => self.add8(bus, op::Reg8::A, op::Reg8::H, true, 1, 4),
            0x8D => self.add8(bus, op::Reg8::A, op::Reg8::L, true, 1, 4),
            0x8E => self.add8(bus, op::Reg8::A, op::Ind8(op::Reg16::HL), true, 1, 7),
            0x8F => self.add8(bus, op::Reg8::A, op::Reg8::A, true, 1, 4),

            0x90 => self.sub8(bus, op::Reg8::A, op::Reg8::B, false, 1, 4),
            0x91 => self.sub8(bus, op::Reg8::A, op::Reg8::C, false, 1, 4),
            0x92 => self.sub8(bus, op::Reg8::A, op::Reg8::D, false, 1, 4),
            0x93 => self.sub8(bus, op::Reg8::A, op::Reg8::E, false, 1, 4),
            0x94 => self.sub8(bus, op::Reg8::A, op::Reg8::H, false, 1, 4),
            0x95 => self.sub8(bus, op::Reg8::A, op::Reg8::L, false, 1, 4),
            0x96 => self.sub8(bus, op::Reg8::A, op::Ind8(op::Reg16::HL), false, 1, 7),
            0x97 => self.sub8(bus, op::Reg8::A, op::Reg8::A, false, 1, 4),
            0x98 => self.sub8(bus, op::Reg8::A, op::Reg8::B, true, 1, 4),
            0x99 => self.sub8(bus, op::Reg8::A, op::Reg8::C, true, 1, 4),
            0x9A => self.sub8(bus, op::Reg8::A, op::Reg8::D, true, 1, 4),
            0x9B => self.sub8(bus, op::Reg8::A, op::Reg8::E, true, 1, 4),
            0x9C => self.sub8(bus, op::Reg8::A, op::Reg8::H, true, 1, 4),
            0x9D => self.sub8(bus, op::Reg8::A, op::Reg8::L, true, 1, 4),
            0x9E => self.sub8(bus, op::Reg8::A, op::Ind8(op::Reg16::HL), true, 1, 7),
            0x9F => self.sub8(bus, op::Reg8::A, op::Reg8::A, true, 1, 4),

            0xA0 => self.and(bus, op::Reg8::A, op::Reg8::B, 1, 4),
            0xA1 => self.and(bus, op::Reg8::A, op::Reg8::C, 1, 4),
            0xA2 => self.and(bus, op::Reg8::A, op::Reg8::D, 1, 4),
            0xA3 => self.and(bus, op::Reg8::A, op::Reg8::E, 1, 4),
            0xA4 => self.and(bus, op::Reg8::A, op::Reg8::H, 1, 4),
            0xA5 => self.and(bus, op::Reg8::A, op::Reg8::L, 1, 4),
            0xA6 => self.and(bus, op::Reg8::A, op::Ind8(op::Reg16::HL), 1, 7),
            0xA7 => self.and(bus, op::Reg8::A, op::Reg8::A,  1, 4),
            0xA8 => self.xor(bus, op::Reg8::A, op::Reg8::B, 1, 4),
            0xA9 => self.xor(bus, op::Reg8::A, op::Reg8::C, 1, 4),
            0xAA => self.xor(bus, op::Reg8::A, op::Reg8::D, 1, 4),
            0xAB => self.xor(bus, op::Reg8::A, op::Reg8::E, 1, 4),
            0xAC => self.xor(bus, op::Reg8::A, op::Reg8::H, 1, 4),
            0xAD => self.xor(bus, op::Reg8::A, op::Reg8::L, 1, 4),
            0xAE => self.xor(bus, op::Reg8::A, op::Ind8(op::Reg16::HL), 1, 7),
            0xAF => self.xor(bus, op::Reg8::A, op::Reg8::A, 1, 4),

            0xB0 => self.or(bus, op::Reg8::A, op::Reg8::B, 1, 4),
            0xB1 => self.or(bus, op::Reg8::A, op::Reg8::C, 1, 4),
            0xB2 => self.or(bus, op::Reg8::A, op::Reg8::D, 1, 4),
            0xB3 => self.or(bus, op::Reg8::A, op::Reg8::E, 1, 4),
            0xB4 => self.or(bus, op::Reg8::A, op::Reg8::H, 1, 4),
            0xB5 => self.or(bus, op::Reg8::A, op::Reg8::L, 1, 4),
            0xB6 => self.or(bus, op::Reg8::A, op::Ind8(op::Reg16::HL), 1, 7),
            0xB7 => self.or(bus, op::Reg8::A, op::Reg8::A,  1, 4),
            0xB8 => self.cp(bus, op::Reg8::A, op::Reg8::B, 1, 4),
            0xB9 => self.cp(bus, op::Reg8::A, op::Reg8::C, 1, 4),
            0xBA => self.cp(bus, op::Reg8::A, op::Reg8::D, 1, 4),
            0xBB => self.cp(bus, op::Reg8::A, op::Reg8::E, 1, 4),
            0xBC => self.cp(bus, op::Reg8::A, op::Reg8::H, 1, 4),
            0xBD => self.cp(bus, op::Reg8::A, op::Reg8::L, 1, 4),
            0xBE => self.cp(bus, op::Reg8::A, op::Ind8(op::Reg16::HL), 1, 7),
            0xBF => self.cp(bus, op::Reg8::A, op::Reg8::A,  1, 4),

            0xC0 => todo!(),
            0xC1 => todo!(),
            0xC2 => todo!(),
            0xC3 => todo!(),
            0xC4 => todo!(),
            0xC5 => todo!(),
            0xC6 => self.add8(bus, op::Reg8::A, op::Imm8::with_offset(1), false, 1, 4),
            0xC7 => todo!(),
            0xC8 => todo!(),
            0xC9 => todo!(),
            0xCA => todo!(),
            0xCB => todo!(),
            0xCC => todo!(),
            0xCD => todo!(),
            0xCE => self.add8(bus, op::Reg8::A, op::Imm8::with_offset(1), true, 1, 4),
            0xCF => todo!(),

            0xD0 => todo!(),
            0xD1 => todo!(),
            0xD2 => todo!(),
            0xD3 => todo!(),
            0xD4 => todo!(),
            0xD5 => todo!(),
            0xD6 => self.sub8(bus, op::Reg8::A, op::Imm8::with_offset(1), false, 1, 4),
            0xD7 => todo!(),
            0xD8 => todo!(),
            0xD9 => self.exx(),
            0xDA => todo!(),
            0xDB => todo!(),
            0xDC => todo!(),
            0xDD => todo!(),
            0xDE => self.sub8(bus, op::Reg8::A, op::Imm8::with_offset(1), true, 1, 4),
            0xDF => todo!(),

            0xE0 => todo!(),
            0xE1 => todo!(),
            0xE2 => todo!(),
            0xE3 => self.ex(bus, op::Ind16(op::Reg16::SP), op::Reg16::HL, 1, 19),
            0xE4 => todo!(),
            0xE5 => todo!(),
            0xE6 => todo!(),
            0xE7 => todo!(),
            0xE8 => todo!(),
            0xE9 => todo!(),
            0xEA => todo!(),
            0xEB => self.ex(bus, op::Reg16::DE, op::Reg16::HL, 1, 4),
            0xEC => todo!(),
            0xED => todo!(),
            0xEE => todo!(),
            0xEF => todo!(),
            
            0xF0 => todo!(),
            0xF1 => todo!(),
            0xF2 => todo!(),
            0xF3 => todo!(),
            0xF4 => todo!(),
            0xF5 => todo!(),
            0xF6 => todo!(),
            0xF7 => todo!(),
            0xF8 => todo!(),
            0xF9 => self.ld(bus, op::Reg16::SP, op::Reg16::HL, 1, 6),
            0xFA => todo!(),
            0xFB => todo!(),
            0xFC => todo!(),
            0xFD => todo!(),
            0xFE => todo!(),
            0xFF => todo!(),
        }
    }

    fn add8<B, D, S> (&mut self, bus: &mut B, dst: D, src: S, with_carry: bool, size: usize, cycles: usize) 
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

    fn add16<B, D, S> (&mut self, bus: &mut B, dst: D, src: S, size: usize, cycles: usize) 
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

    fn and<B, D, S> (&mut self, bus: &mut B, dst: D, src: S, size: usize, cycles: usize) 
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

    fn ccf(&mut self) {
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

    fn cp<B, D, S> (&mut self, bus: &mut B, dst: D, src: S, size: usize, cycles: usize) 
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

    fn cpl(&mut self) {
        let a = self.regs.a();
        let c = !a;
        self.regs.set_a(c);

        self.regs.update_flags(flag::intrinsic_undocumented(c) + flag::N + flag::H);

        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn daa(&mut self) {
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

    fn dec8<B, D> (&mut self, bus: &mut B, dst: D, size: usize, cycles: usize) 
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

    fn dec16<B, D> (&mut self, bus: &mut B, dst: D, size: usize, cycles: usize) 
    where B: Bus, D: op::DestOp<u16> {
        let mut ctx = op::Context::from(bus, &mut self.regs);
        let val = dst.get(&ctx) - 1;
        dst.set(&mut ctx, val);
        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn djnz<B: Bus>(&mut self, bus: &mut B) {
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

    fn ex<B, D1, D2> (&mut self, bus: &mut B, opa: D1, opb: D2, size: usize, cycles: usize) 
    where B: Bus, D1: op::DestOp<u16>, D2: op::DestOp<u16> {
        let mut ctx = op::Context::from(bus, &mut self.regs);
        let a = opa.get(&ctx);
        let b = opb.get(&ctx);

        opa.set(&mut ctx, b);
        opb.set(&mut ctx, a);

        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn exx(&mut self){
        self.regs.swap_bc();
        self.regs.swap_de();
        self.regs.swap_hl();

        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn inc8<B, D> (&mut self, bus: &mut B, dst: D, size: usize, cycles: usize) 
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

    fn inc16<B, D> (&mut self, bus: &mut B, dst: D, size: usize, cycles: usize) 
    where B: Bus, D: op::DestOp<u16> {
        let mut ctx = op::Context::from(bus, &mut self.regs);
        let val = dst.get(&ctx) + 1;
        dst.set(&mut ctx, val);
        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn jr<B: Bus, P: flag::Predicate>(&mut self, bus: &mut B, pred: P) {
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

    fn ld<T, B, D, S> (&mut self, bus: &mut B, dst: D, src: S, size: usize, cycles: usize) 
    where B: Bus, D: op::DestOp<T>, S: op::SrcOp<T> {
        let mut ctx = op::Context::from(bus, &mut self.regs);
        let val = src.get(&ctx);
        dst.set(&mut ctx, val);
        self.regs.inc_pc(size);
        self.cycles += cycles;
    }

    fn nop(&mut self) {
        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn or<B, D, S> (&mut self, bus: &mut B, dst: D, src: S, size: usize, cycles: usize) 
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

    fn rla(&mut self) {
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

    fn rlca(&mut self) {
        let a = self.regs.a();
        let c = (a << 1) | (a >> 7);

        self.regs.set_a(c);

        self.regs.update_flags(
            flag::intrinsic_undocumented(c) & flag::C.on(a & 0x80 > 0) - flag::H - flag::N
        );

        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn rra(&mut self) {
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

    fn rrca(&mut self) {
        let a = self.regs.a();
        let c = (a >> 1) | (a << 7);

        self.regs.set_a(c);

        self.regs.update_flags(
            flag::intrinsic_undocumented(c) & flag::C.on(a & 0x01 > 0) - flag::H - flag::N
        );

        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn scf(&mut self) {
        self.regs.update_flags(
            flag::intrinsic_undocumented(self.regs.a()) + flag::C - flag::N - flag::H
        );
        self.regs.inc_pc(1);
        self.cycles += 4;
    }

    fn sub8<B, D, S> (&mut self, bus: &mut B, dst: D, src: S, with_carry: bool, size: usize, cycles: usize) 
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

    fn xor<B, D, S> (&mut self, bus: &mut B, dst: D, src: S, size: usize, cycles: usize) 
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
}
