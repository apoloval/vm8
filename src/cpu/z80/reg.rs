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

impl Register {
    pub fn low(&self) -> u8 { unsafe { self.as_byte.l } }
    pub fn high(&self) -> u8 { unsafe { self.as_byte.h } }
    
    pub fn set_low(&mut self, val: u8) { unsafe { self.as_byte.l = val } }
    pub fn set_high(&mut self, val: u8) { unsafe { self.as_byte.h = val } }
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
    ix: Register,
    iy: Register,

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

    #[inline] pub fn af_(&self) -> u16 { *self.af_ }
    #[inline] pub fn bc_(&self) -> u16 { *self.bc_ }
    #[inline] pub fn de_(&self) -> u16 { *self.de_ }
    #[inline] pub fn hl_(&self) -> u16 { *self.hl_ }

    #[inline] pub fn set_af_(&mut self, val: u16) { *self.af_ = val }
    #[inline] pub fn set_bc_(&mut self, val: u16) { *self.bc_ = val  }
    #[inline] pub fn set_de_(&mut self, val: u16) { *self.de_ = val  }
    #[inline] pub fn set_hl_(&mut self, val: u16) { *self.hl_ = val  }

    #[inline] pub fn flags(&self) -> u8 { self.af.low() }
    #[inline] pub fn pc(&self) -> u16 { *self.pc }
    #[inline] pub fn sp(&self) -> u16 { *self.sp }

    #[inline] pub fn set_flags(&mut self, val: u8) { self.af.set_low(val) }
    #[inline] pub fn set_pc(&mut self, val: u16) { *self.pc = val }
    #[inline] pub fn set_sp(&mut self, val: u16) { *self.sp = val }

    #[inline] pub fn flag_s(&self) -> u8 { flag!(S, self.flags()) }
    #[inline] pub fn flag_z(&self) -> u8 { flag!(Z, self.flags()) }
    #[inline] pub fn flag_h(&self) -> u8 { flag!(H, self.flags()) }
    #[inline] pub fn flag_pv(&self) -> u8 { flag!(PV, self.flags()) }
    #[inline] pub fn flag_n(&self) -> u8 { flag!(N, self.flags()) }
    #[inline] pub fn flag_c(&self) -> u8 { flag!(C, self.flags()) }

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

    #[inline] pub fn inc_pc(&mut self, val: usize) { *self.pc += val as u16 }
    #[inline] pub fn inc_pc8(&mut self, val: u8) { self.inc_pc(val as i8 as usize) }
}

macro_rules! reg_read {
    ($cpu:expr, A) => { $cpu.regs().a() };
    ($cpu:expr, B) => { $cpu.regs().b() };
    ($cpu:expr, C) => { $cpu.regs().c() };
    ($cpu:expr, D) => { $cpu.regs().d() };
    ($cpu:expr, E) => { $cpu.regs().e() };
    ($cpu:expr, H) => { $cpu.regs().h() };
    ($cpu:expr, L) => { $cpu.regs().l() };
    ($cpu:expr, BC) => { $cpu.regs().bc() };
    ($cpu:expr, DE) => { $cpu.regs().de() };
    ($cpu:expr, HL) => { $cpu.regs().hl() };
    ($cpu:expr, ($reg:tt)) => { $cpu.mem().read_from(reg_read!($cpu, $reg)) };
    ($cpu:expr, $val:tt) => { $val };
}

macro_rules! reg_write {
    ($cpu:expr, A, $val:expr) => { { $cpu.regs_mut().set_a($val); $val } };
    ($cpu:expr, B, $val:expr) => { { $cpu.regs_mut().set_b($val); $val } };
    ($cpu:expr, C, $val:expr) => { { $cpu.regs_mut().set_c($val); $val } };
    ($cpu:expr, D, $val:expr) => { { $cpu.regs_mut().set_d($val); $val } };
    ($cpu:expr, E, $val:expr) => { { $cpu.regs_mut().set_e($val); $val } };
    ($cpu:expr, H, $val:expr) => { { $cpu.regs_mut().set_h($val); $val } };
    ($cpu:expr, L, $val:expr) => { { $cpu.regs_mut().set_l($val); $val } };
}