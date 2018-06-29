use std::mem;

use bus::Address;

pub trait Register<T> {
    fn read(&self, regs: &Registers) -> T;
    fn write(&self, regs: &mut Registers, val: T);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reg8 { A, B, C, D, E, H, L, IXH, IXL, IYH, IYL }

impl Register<u8> for Reg8 {
    fn read(&self, regs: &Registers) -> u8 {
        match self {
            Reg8::A => Self::read_h(&regs.af),
            Reg8::B => Self::read_h(&regs.bc),
            Reg8::C => Self::read_l(&regs.bc),
            Reg8::D => Self::read_h(&regs.de),
            Reg8::E => Self::read_l(&regs.de),
            Reg8::H => Self::read_h(&regs.hl),
            Reg8::L => Self::read_l(&regs.hl),
            Reg8::IXH => Self::read_h(&regs.ix),
            Reg8::IXL => Self::read_l(&regs.ix),
            Reg8::IYH => Self::read_h(&regs.iy),
            Reg8::IYL => Self::read_l(&regs.iy),
        }
    }

    fn write(&self, regs: &mut Registers, val: u8) {
        match self {
            Reg8::A => Self::write_h(&mut regs.af, val),
            Reg8::B => Self::write_h(&mut regs.bc, val),
            Reg8::C => Self::write_l(&mut regs.bc, val),
            Reg8::D => Self::write_h(&mut regs.de, val),
            Reg8::E => Self::write_l(&mut regs.de, val),
            Reg8::H => Self::write_h(&mut regs.hl, val),
            Reg8::L => Self::write_l(&mut regs.hl, val),
            Reg8::IXH => Self::write_h(&mut regs.ix, val),
            Reg8::IXL => Self::write_l(&mut regs.ix, val),
            Reg8::IYH => Self::write_h(&mut regs.iy, val),
            Reg8::IYL => Self::write_l(&mut regs.iy, val),
        }
    }
}

impl Reg8 {
    fn read_h(src: &u16) -> u8 {
        (*src >> 8) as u8
    }

    fn read_l(src: &u16) -> u8 {
        *src as u8
    }

    fn write_h(dst: &mut u16, val: u8) {
        *dst = (*dst & 0x00ff) | ((val as u16) << 8);
    }

    fn write_l(dst: &mut u16, val: u8) {
        *dst = (*dst & 0x00ff) | (val as u16);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

    pub fn pc(&self) -> Address {
        return Address::from(self.pc)
    }

    pub fn inc_pc(&mut self, val: usize) {
        self.pc += val as u16
    }
}