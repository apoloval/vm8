use std::mem;
use std::ops::{Deref, DerefMut};

use bus::Address;

pub trait Read<T> {
    fn read(&self, regs: &Registers) -> T;
}

pub trait Write<T> {
    fn write(&self, regs: &mut Registers, val: T);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Name8 { A, B, C, D, E, H, L, IXH, IXL, IYH, IYL }

impl Read<u8> for Name8 {
    fn read(&self, regs: &Registers) -> u8 {
        unsafe {
            match self {
                Name8::A => regs.af.as_byte.h,
                Name8::B => regs.bc.as_byte.h,
                Name8::C => regs.bc.as_byte.l,
                Name8::D => regs.de.as_byte.h,
                Name8::E => regs.de.as_byte.l,
                Name8::H => regs.hl.as_byte.h,
                Name8::L => regs.hl.as_byte.l,
                Name8::IXH => regs.ix.as_byte.h,
                Name8::IXL => regs.ix.as_byte.l,
                Name8::IYH => regs.iy.as_byte.h,
                Name8::IYL => regs.iy.as_byte.l,
            }
        }
    }
}

impl Write<u8> for Name8 {
    fn write(&self, regs: &mut Registers, val: u8) {
        unsafe {
            match self {
                Name8::A => regs.af.as_byte.h = val,
                Name8::B => regs.bc.as_byte.h = val,
                Name8::C => regs.bc.as_byte.l = val,
                Name8::D => regs.de.as_byte.h = val,
                Name8::E => regs.de.as_byte.l = val,
                Name8::H => regs.hl.as_byte.h = val,
                Name8::L => regs.hl.as_byte.l = val,
                Name8::IXH => regs.ix.as_byte.h = val,
                Name8::IXL => regs.ix.as_byte.l = val,
                Name8::IYH => regs.iy.as_byte.h = val,
                Name8::IYL => regs.iy.as_byte.l = val,
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Name16 { AF, BC, DE, HL, IX, IY }

impl Read<u16> for Name16 {
    fn read(&self, regs: &Registers) -> u16 {
        match self {
            Name16::AF => *regs.af,
            Name16::BC => *regs.bc,
            Name16::DE => *regs.de,
            Name16::HL => *regs.hl,
            Name16::IX => *regs.ix,
            Name16::IY => *regs.iy,
        }
    }
}

impl Write<u16> for Name16 {
    fn write(&self, regs: &mut Registers, val: u16) {
        match self {
            Name16::AF => *regs.af = val,
            Name16::BC => *regs.bc = val,
            Name16::DE => *regs.de = val,
            Name16::HL => *regs.hl = val,
            Name16::IX => *regs.ix = val,
            Name16::IY => *regs.iy = val,
        }
    }
}

pub struct FlagUpdate {
    set: u8,
    reset: u8,
}

impl FlagUpdate {
    pub fn new(opcode: u8) -> FlagUpdate { 
        FlagUpdate{set: opcode & 0b00101000, reset: 0xff} 
    }

    #[allow(non_snake_case)]
    pub fn C(self, val: u8) -> FlagUpdate { self.bit(0, val) }
    
    #[allow(non_snake_case)]
    pub fn N(self, val: u8) -> FlagUpdate { self.bit(1, val) }
    
    #[allow(non_snake_case)]
    pub fn PV(self, val: u8) -> FlagUpdate { self.bit(2, val) }
    
    #[allow(non_snake_case)]
    pub fn H(self, val: u8) -> FlagUpdate { self.bit(4, val) }
    
    #[allow(non_snake_case)]
    pub fn Z(self, val: u8) -> FlagUpdate { self.bit(6, val) }

    #[allow(non_snake_case)]
    pub fn S(self, val: u8) -> FlagUpdate { self.bit(7, val) }

    pub fn apply(&self, flags: &mut u8) {
        *flags |= self.set;
        *flags &= self.reset;
    }

    fn bit(self, bit: i8, val: u8) -> FlagUpdate {
        if val != 0 { FlagUpdate { set: self.set | (1 << bit), reset: self.reset } }
        else { FlagUpdate { set: self.set, reset: self.reset & (!(1 << bit)) } }
    }
}

#[derive(Clone, Copy)]
pub struct Name8Pair {
    l: u8,
    h: u8,
}

#[derive(Clone, Copy)]
pub union Register {
    word: u16,
    as_byte: Name8Pair,
}

impl Default for Register {
    fn default() -> Register {
        Register { word: 0 }
    }
}

impl Deref for Register {
    type Target = u16;
    fn deref(&self) -> &u16 {
        return unsafe { &self.word };
    }
}

impl DerefMut for Register {
    fn deref_mut(&mut self) -> &mut u16 {
        return unsafe { &mut self.word };
    }
}

#[derive(Default)]
pub struct Registers {
    // Primary 16-bits registers
    pub af: Register,
    pub bc: Register,
    pub de: Register,
    pub hl: Register,

    // Alternative 16-bits registers
    af_: Register,
    bc_: Register,
    de_: Register,
    hl_: Register,

    // Index registers
    pub ix: Register,
    pub iy: Register,

    // Control registers
    pub sp: Register,
    pub pc: Register,
}

impl Registers {
    pub fn new() -> Registers {
        Self::default()
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

    pub fn update_flags(&mut self, update: FlagUpdate) {
        unsafe {
            let mut val = self.af.as_byte.l;
            update.apply(&mut val);
            self.af.as_byte.l &= val;
        }
    }

    pub fn pc(&self) -> Address { Address::from(*self.pc) }
    pub fn set_pc(&mut self, addr: Address) { *self.pc = u16::from(addr) }
    pub fn inc_pc(&mut self, val: usize) { *self.pc += val as u16 }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_flag_update() {
        let mut flags: u8 = 0;
        FlagUpdate::new(0b00101000).C(1).apply(&mut flags);
        assert_eq!(0b00101001, flags);
        FlagUpdate::new(0b00101000).N(1).apply(&mut flags);
        assert_eq!(0b00101011, flags);
        FlagUpdate::new(0b00101000).PV(1).apply(&mut flags);
        assert_eq!(0b00101111, flags);
        FlagUpdate::new(0b00101000).H(1).apply(&mut flags);
        assert_eq!(0b00111111, flags);
        FlagUpdate::new(0b00101000).Z(1).apply(&mut flags);
        assert_eq!(0b01111111, flags);
        FlagUpdate::new(0b00101000).S(1).apply(&mut flags);
        assert_eq!(0b11111111, flags);

        FlagUpdate::new(0b00101000).C(0).apply(&mut flags);
        assert_eq!(0b11111110, flags);
        FlagUpdate::new(0b00101000).N(0).apply(&mut flags);
        assert_eq!(0b11111100, flags);
        FlagUpdate::new(0b00101000).PV(0).apply(&mut flags);
        assert_eq!(0b11111000, flags);
        FlagUpdate::new(0b00101000).H(0).apply(&mut flags);
        assert_eq!(0b11101000, flags);
        FlagUpdate::new(0b00101000).Z(0).apply(&mut flags);
        assert_eq!(0b10101000, flags);
        FlagUpdate::new(0b00101000).S(0).apply(&mut flags);
        assert_eq!(0b00101000, flags);
    }
}