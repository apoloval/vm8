use crate::cpu::z80::MemAddr;

#[derive(Default)]
pub struct RegBank {
  pub af: PairedReg,
  pub bc: PairedReg,
  pub de: PairedReg,
  pub hl: PairedReg,

  pub pc: Reg16,
  pub sp: Reg16,
}

impl RegBank {
  pub fn pc_addr(&self) -> MemAddr {
    MemAddr(self.pc)
  }

  pub fn inc_pc(&mut self, n: u16) {
    self.pc = u16::from(self.pc_addr() + n);
  }
}

#[derive(Clone, Copy, Default)]
pub struct Reg8 {
  pub l: u8,
  pub h: u8,
}

pub type Reg16 = u16;

#[derive(Clone, Copy)]
pub union PairedReg {
  r8: Reg8,
  r16: Reg16,
}

impl PairedReg {
  pub fn r8(&self) -> Reg8 {
    unsafe { self.r8 }
  }
  pub fn r16(&self) -> Reg16 {
    unsafe { self.r16 }
  }
  pub fn r8_mut(&mut self) -> &mut Reg8 {
    unsafe { &mut self.r8 }
  }
  pub fn r16_mut(&mut self) -> &mut Reg16 {
    unsafe { &mut self.r16 }
  }
}

impl Default for PairedReg {
  fn default() -> PairedReg {
    PairedReg { r16: 0 }
  }
}
