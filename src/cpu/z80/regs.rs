use std::mem;

use bus::Addr16;

pub trait Register<T> {
    fn read(&self, regs: &Registers) -> T;
    fn write(&self, regs: &mut Registers, val: T);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reg8 { A, B, C, D, E, H, L, IXH, IXL, IYH, IYL }

impl Register<u8> for Reg8 {
    fn read(&self, regs: &Registers) -> u8 {
        match self {
            Reg8::A => (regs.af >> 8) as u8,
            Reg8::B => (regs.bc >> 8) as u8,
            Reg8::C => (regs.bc) as u8,
            Reg8::D => (regs.de >> 8) as u8,
            Reg8::E => (regs.de) as u8,
            Reg8::H => (regs.hl >> 8) as u8,
            Reg8::L => (regs.hl) as u8,
            Reg8::IXH => (regs.ix >> 8) as u8,
            Reg8::IXL => (regs.ix) as u8,
            Reg8::IYH => (regs.iy >> 8) as u8,
            Reg8::IYL => (regs.iy) as u8,
        }
    }

    fn write(&self, regs: &mut Registers, val: u8) {
        match self {
            Reg8::A => regs.af = (regs.af & 0x00ff) | ((val as u16) << 8),
            Reg8::B => regs.bc = (regs.bc & 0x00ff) | ((val as u16) << 8),
            Reg8::C => regs.bc = (regs.bc & 0xff00) | (val as u16),
            Reg8::D => regs.de = (regs.de & 0x00ff) | ((val as u16) << 8),
            Reg8::E => regs.de = (regs.de & 0xff00) | (val as u16),
            Reg8::H => regs.hl = (regs.hl & 0x00ff) | ((val as u16) << 8),
            Reg8::L => regs.hl = (regs.hl & 0xff00) | (val as u16),
            Reg8::IXH => regs.ix = (regs.ix & 0x00ff) | ((val as u16) << 8),
            Reg8::IXL => regs.ix = (regs.ix & 0xff00) | (val as u16),
            Reg8::IYH => regs.iy = (regs.iy & 0x00ff) | ((val as u16) << 8),
            Reg8::IYL => regs.iy = (regs.iy & 0xff00) | (val as u16),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reg16 { AF, BC, DE, HL, IX, IY }

impl Register<u16> for Reg16 {
    fn read(&self, regs: &Registers) -> u16 {
        match self {
            Reg16::AF => regs.af,
            Reg16::BC => regs.bc,
            Reg16::DE => regs.de,
            Reg16::HL => regs.hl,
            Reg16::IX => regs.ix,
            Reg16::IY => regs.iy,
        }
    }

    fn write(&self, regs: &mut Registers, val: u16) {
        match self {
            Reg16::AF => regs.af = val,
            Reg16::BC => regs.bc = val,
            Reg16::DE => regs.de = val,
            Reg16::HL => regs.hl = val,
            Reg16::IX => regs.ix = val,
            Reg16::IY => regs.iy = val,
        }
    }
}

pub struct Registers {
    // Primary 16-bits registers
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,

    // Alternative 16-bits registers
    af_: u16,
    bc_: u16,
    de_: u16,
    hl_: u16,

    // Index registers
    pub ix: u16,
    pub iy: u16,

    // Control registers
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    pub fn new() -> Registers {
        Registers { 
            af: 0, bc: 0, de: 0, hl: 0,
            af_: 0, bc_: 0, de_: 0, hl_: 0,
            ix:0, iy: 0,
            sp:0, pc: 0,
        }
    }

    // Swap the primary and alternative registers AF/AF'
    pub fn swap_af(&mut self) {
        mem::swap(&mut self.af, &mut self.af_);
    }

    // Swap the primary and alternative registers BC DE HL/AF' DE' HL'
    pub fn swap(&mut self) {
        mem::swap(&mut self.bc, &mut self.bc_);
        mem::swap(&mut self.de, &mut self.de_);
        mem::swap(&mut self.hl, &mut self.hl_);
    }

    pub fn pc(&self) -> Addr16 {
        return Addr16::from(self.pc)
    }

    pub fn inc_pc(&mut self, val: usize) {
        self.pc += val as u16
    }
}