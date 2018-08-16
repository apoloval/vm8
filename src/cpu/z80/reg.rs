use std::mem;
use std::ops::{Deref, DerefMut};

use bus::Address;

pub struct FlagUpdate {
    set: u8,
    reset: u8,
}

impl FlagUpdate {
    pub fn new() -> FlagUpdate {
        FlagUpdate{set: 0x00, reset: 0xff}
    }

    pub fn with_opcode(opcode: u32) -> FlagUpdate { 
        FlagUpdate{set: (opcode as u8) & 0b00101000, reset: 0xff} 
    }

    #[allow(non_snake_case)]
    pub fn C(self, val: bool) -> FlagUpdate { self.bit(0, val) }
    
    #[allow(non_snake_case)]
    pub fn N(self, val: bool) -> FlagUpdate { self.bit(1, val) }
    
    #[allow(non_snake_case)]
    pub fn PV(self, val: bool) -> FlagUpdate { self.bit(2, val) }
    
    #[allow(non_snake_case)]
    pub fn H(self, val: bool) -> FlagUpdate { self.bit(4, val) }
    
    #[allow(non_snake_case)]
    pub fn Z(self, val: bool) -> FlagUpdate { self.bit(6, val) }

    #[allow(non_snake_case)]
    pub fn S(self, val: bool) -> FlagUpdate { self.bit(7, val) }

    pub fn apply(&self, flags: &mut u8) {
        *flags |= self.set;
        *flags &= self.reset;
    }

    fn bit(self, bit: i8, val: bool) -> FlagUpdate {
        if val { FlagUpdate { set: self.set | (1 << bit), reset: self.reset } }
        else { FlagUpdate { set: self.set, reset: self.reset & (!(1 << bit)) } }
    }
}

#[derive(Clone, Copy)]
pub struct Name8Pair {
    pub l: u8,
    pub h: u8,
}

#[derive(Clone, Copy)]
pub union Register {
    pub word: u16,
    pub as_byte: Name8Pair,
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
        FlagUpdate::with_opcode(0b00101000).C(true).apply(&mut flags);
        assert_eq!(0b00101001, flags);
        FlagUpdate::with_opcode(0b00101000).N(true).apply(&mut flags);
        assert_eq!(0b00101011, flags);
        FlagUpdate::with_opcode(0b00101000).PV(true).apply(&mut flags);
        assert_eq!(0b00101111, flags);
        FlagUpdate::with_opcode(0b00101000).H(true).apply(&mut flags);
        assert_eq!(0b00111111, flags);
        FlagUpdate::with_opcode(0b00101000).Z(true).apply(&mut flags);
        assert_eq!(0b01111111, flags);
        FlagUpdate::with_opcode(0b00101000).S(true).apply(&mut flags);
        assert_eq!(0b11111111, flags);

        FlagUpdate::with_opcode(0b00101000).C(false).apply(&mut flags);
        assert_eq!(0b11111110, flags);
        FlagUpdate::with_opcode(0b00101000).N(false).apply(&mut flags);
        assert_eq!(0b11111100, flags);
        FlagUpdate::with_opcode(0b00101000).PV(false).apply(&mut flags);
        assert_eq!(0b11111000, flags);
        FlagUpdate::with_opcode(0b00101000).H(false).apply(&mut flags);
        assert_eq!(0b11101000, flags);
        FlagUpdate::with_opcode(0b00101000).Z(false).apply(&mut flags);
        assert_eq!(0b10101000, flags);
        FlagUpdate::with_opcode(0b00101000).S(false).apply(&mut flags);
        assert_eq!(0b00101000, flags);
    }
}