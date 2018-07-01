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
        *dst = (*dst & 0xff00) | (val as u16);
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

    pub fn update_flags(&mut self, update: FlagUpdate) {
        let mut val = self.af as u8;
        update.apply(&mut val);
        self.af = (self.af & 0xff00) | (val as u16);
    }

    pub fn pc(&self) -> Address { Address::from(self.pc) }
    pub fn set_pc(&mut self, addr: Address) { self.pc = u16::from(addr) }
    pub fn inc_pc(&mut self, val: usize) { self.pc += val as u16 }
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