use std::mem;
use std::ops::{Deref, DerefMut};

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
        unsafe { &self.word }
    }
}

impl DerefMut for Register {
    fn deref_mut(&mut self) -> &mut u16 {
        unsafe { &mut self.word }
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
    #[inline]
    pub fn swap_af(&mut self) {
        mem::swap(&mut self.af, &mut self.af_);
    }

    // Swap the primary and alternative registers BC DE HL/AF' DE' HL'
    #[inline]
    pub fn swap(&mut self) {
        mem::swap(&mut self.bc, &mut self.bc_);
        mem::swap(&mut self.de, &mut self.de_);
        mem::swap(&mut self.hl, &mut self.hl_);
    }

    #[inline]
    pub fn flags(&self) -> u8 {
        unsafe { self.af.as_byte.l }
    }

    #[inline]
    pub fn set_flags(&mut self, val: u8) {
        unsafe { self.af.as_byte.l = val; }
    }

    #[inline]
    pub fn inc_pc(&mut self, val: usize) { *self.pc += val as u16 }
}
