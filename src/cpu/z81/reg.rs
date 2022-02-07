use std::mem;
use std::ops::{Deref, DerefMut};

use crate::cpu::z81::flag;

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

impl Register {
    pub fn low(&self) -> u8 { unsafe { self.as_byte.l } }
    pub fn high(&self) -> u8 { unsafe { self.as_byte.h } }

    pub fn set_low(&mut self, val: u8) { self.as_byte.l = val }
    pub fn set_high(&mut self, val: u8) { self.as_byte.h = val }
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
    af: Register,
    bc: Register,
    de: Register,
    hl: Register,

    // Alternative 16-bits registers
    af_: Register,
    bc_: Register,
    de_: Register,
    hl_: Register,

    // Index registers
    _ix: Register,
    _iy: Register,

    // Control registers
    sp: Register,
    pc: Register,
}

impl Registers {
    pub fn new() -> Registers {
        Self::default()
    }

    #[inline] pub fn af(&self) -> u16 { *self.af }
    #[inline] pub fn bc(&self) -> u16 { *self.bc }
    #[inline] pub fn de(&self) -> u16 { *self.de }
    #[inline] pub fn hl(&self) -> u16 { *self.hl }

    #[inline] pub fn set_af(&mut self, val: u16) { *self.af = val }
    #[inline] pub fn set_bc(&mut self, val: u16) { *self.bc = val  }
    #[inline] pub fn set_de(&mut self, val: u16) { *self.de = val  }
    #[inline] pub fn set_hl(&mut self, val: u16) { *self.hl = val  }

    #[inline] pub fn a(&self) -> u8 { self.af.high() }
    #[inline] pub fn b(&self) -> u8 { self.bc.high() }
    #[inline] pub fn c(&self) -> u8 { self.bc.low() }
    #[inline] pub fn d(&self) -> u8 { self.de.high() }
    #[inline] pub fn e(&self) -> u8 { self.de.low() }
    #[inline] pub fn h(&self) -> u8 { self.hl.high() }
    #[inline] pub fn l(&self) -> u8 { self.hl.low() }

    #[inline] pub fn set_a(&mut self, val: u8) { self.af.set_high(val) }
    #[inline] pub fn set_b(&mut self, val: u8) { self.bc.set_high(val) }
    #[inline] pub fn set_c(&mut self, val: u8) { self.bc.set_low(val) }
    #[inline] pub fn set_d(&mut self, val: u8) { self.de.set_high(val) }
    #[inline] pub fn set_e(&mut self, val: u8) { self.de.set_low(val) }
    #[inline] pub fn set_h(&mut self, val: u8) { self.hl.set_high(val) }
    #[inline] pub fn set_l(&mut self, val: u8) { self.hl.set_low(val) }

    #[inline] #[cfg(test)] pub fn af_(&self) -> u16 { *self.af_ }
    #[inline] #[cfg(test)] pub fn bc_(&self) -> u16 { *self.bc_ }
    #[inline] #[cfg(test)] pub fn de_(&self) -> u16 { *self.de_ }
    #[inline] #[cfg(test)] pub fn hl_(&self) -> u16 { *self.hl_ }

    #[inline] #[cfg(test)] pub fn set_af_(&mut self, val: u16) { *self.af_ = val }
    #[inline] #[cfg(test)] pub fn set_bc_(&mut self, val: u16) { *self.bc_ = val }
    #[inline] #[cfg(test)] pub fn set_de_(&mut self, val: u16) { *self.de_ = val }
    #[inline] #[cfg(test)] pub fn set_hl_(&mut self, val: u16) { *self.hl_ = val }

    #[inline] pub fn flags(&self) -> u8 { self.af.low() }
    #[inline] pub fn pc(&self) -> u16 { *self.pc }
    #[inline] pub fn sp(&self) -> u16 { *self.sp }

    #[inline] pub fn set_flags(&mut self, val: u8) { self.af.set_low(val) }
    #[inline] pub fn set_pc(&mut self, val: u16) { *self.pc = val }
    #[inline] pub fn set_sp(&mut self, val: u16) { *self.sp = val }

    #[inline] pub fn swap_af(&mut self) { mem::swap(&mut self.af, &mut self.af_); }
    #[inline] pub fn swap_bc(&mut self) { mem::swap(&mut self.bc, &mut self.bc_); }
    #[inline] pub fn swap_de(&mut self) { mem::swap(&mut self.de, &mut self.de_); }
    #[inline] pub fn swap_hl(&mut self) { mem::swap(&mut self.hl, &mut self.hl_); }

    #[inline] pub fn inc_pc(&mut self, val: usize) -> u16 { *self.pc += val as u16; *self.pc }
    #[inline] pub fn inc_pc8(&mut self, val: u8) -> u16 { self.inc_pc(val as i8 as usize) }

    #[inline] pub fn inc_sp(&mut self, val: usize) -> u16 { *self.sp += val as u16; *self.sp }
    #[inline] pub fn dec_sp(&mut self, val: usize) -> u16 { *self.sp -= val as u16; *self.sp }

    #[inline] pub fn flag(&self, f: flag::Flag) -> bool { f.check(self.flags()) }

    #[inline] 
    pub fn update_flags(&mut self, aff: flag::Affection) {
        let mut f = self.flags();
        f = aff.apply(f);
        self.set_flags(f);
    }
}
