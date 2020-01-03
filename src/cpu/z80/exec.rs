use std::num::Wrapping;

use crate::emu::Cycles;
use crate::cpu::z80::{MemBus, IOBus};
use crate::cpu::z80::regs::RegBank;

// The context in which the CPU will execute instructions.
pub trait Context {
  type Mem: MemBus;
  type IO: IOBus;

  fn regs(&self) -> &RegBank;
  fn regs_mut(&mut self) -> &mut RegBank;
  fn mem(&self) -> &Self::Mem;
  fn mem_mut(&mut self) -> &mut Self::Mem;
  fn io(&self) -> &Self::IO;
  fn io_mut(&mut self) -> &mut Self::IO;

  // Skip the opcode byte and read a 8-bit operand from PC+1
  fn read_op8(&self) -> u8 {
    let Wrapping(addr) = Wrapping(self.regs().pc) + Wrapping(1);
    self.mem().mem_read(addr)
  }

  // Skip the opcode byte and read a 16-bit operand from PC+1
  fn read_op16(&self) -> u16 {
    let Wrapping(addr) = Wrapping(self.regs().pc) + Wrapping(1);
    self.mem().mem_read16(addr)
  }
}

// An operand of a Z80 instruction
pub trait Operand {
  // Return the number of cycles required to fetch in/out the operand value.
  fn cycles() -> Cycles;

  // Return the size this operand occupies in the instruction.
  fn size() -> u16;
}

// A source operand for 8-bit operations.
pub trait Src8 : Operand {
  fn load<C: Context>(&self, ctx: &C) -> u8;
}

// A destination operand for 8-bit operations.
pub trait Dst8 : Operand  {
  fn store<C: Context>(&self, ctx: &mut C, val: u8);
}

// A source operand for 16-bit operations.
pub trait Src16 : Operand {
  fn load<C: Context>(&self, ctx: &C) -> u16;
}

// A destination operand for 16-bit operations.
pub trait Dst16 : Operand  {
  fn store<C: Context>(&self, ctx: &mut C, val: u16);
}


// An 8-bit register direct operand.
pub enum Reg8 { A, B, C, D, E, H, L }

impl Operand for Reg8 {
  fn cycles() -> Cycles { Cycles(0) }
  fn size() -> u16 { 0 }
}

impl Src8 for Reg8 {
  fn load<C: Context>(&self, ctx: &C) -> u8 {
    match self {
      Reg8::A => ctx.regs().af.r8().h,
      Reg8::B => ctx.regs().bc.r8().h,
      Reg8::C => ctx.regs().bc.r8().l,
      Reg8::D => ctx.regs().de.r8().h,
      Reg8::E => ctx.regs().de.r8().l,
      Reg8::H => ctx.regs().hl.r8().h,
      Reg8::L => ctx.regs().hl.r8().l,
    }
  }
}

impl Dst8 for Reg8 {
  fn store<C: Context>(&self, ctx: &mut C, val: u8) {
    match self {
      Reg8::A => ctx.regs_mut().af.r8_mut().h = val,
      Reg8::B => ctx.regs_mut().bc.r8_mut().h = val,
      Reg8::C => ctx.regs_mut().bc.r8_mut().l = val,
      Reg8::D => ctx.regs_mut().de.r8_mut().h = val,
      Reg8::E => ctx.regs_mut().de.r8_mut().l = val,
      Reg8::H => ctx.regs_mut().hl.r8_mut().h = val,
      Reg8::L => ctx.regs_mut().hl.r8_mut().l = val,
    }
  }
}


// An 8-bit register indirect operand.
pub enum IndReg8 { BC, DE, HL }

impl Operand for IndReg8 {
  fn cycles() -> Cycles { Cycles(3) }
  fn size() -> u16 { 0 }
}

impl Src8 for IndReg8 {
  fn load<C: Context>(&self, ctx: &C) -> u8 {
    let addr = match self {
      IndReg8::BC => ctx.regs().bc.r16(),
      IndReg8::DE => ctx.regs().de.r16(),
      IndReg8::HL => ctx.regs().hl.r16(),
    };
    ctx.mem().mem_read(addr)
  }
}

impl Dst8 for IndReg8 {
  fn store<C: Context>(&self, ctx: &mut C, val: u8) {
    let addr = match self {
      IndReg8::BC => ctx.regs().bc.r16(),
      IndReg8::DE => ctx.regs().de.r16(),
      IndReg8::HL => ctx.regs().hl.r16(),
    };
    ctx.mem_mut().mem_write(addr, val);
  }
}


// A 16-bit register direct operand
pub enum Reg16 { BC, DE, HL, SP }

impl Operand for Reg16 {
  fn cycles() -> Cycles { Cycles(0) }
  fn size() -> u16 { 0 }
}

impl Src16 for Reg16 {
  fn load<C: Context>(&self, ctx: &C) -> u16 {
    match self {
      Reg16::BC => ctx.regs().bc.r16(),
      Reg16::DE => ctx.regs().de.r16(),
      Reg16::HL => ctx.regs().hl.r16(),
      Reg16::SP => ctx.regs().sp,
    }
  }
}

impl Dst16 for Reg16 {
  fn store<C: Context>(&self, ctx: &mut C, val: u16) {
    match self {
      Reg16::BC => *ctx.regs_mut().bc.r16_mut() = val,
      Reg16::DE => *ctx.regs_mut().de.r16_mut() = val,
      Reg16::HL => *ctx.regs_mut().hl.r16_mut() = val,
      Reg16::SP => ctx.regs_mut().sp = val,
    }
  }
}


// A 8-bit literal operand
pub struct Liter8;

impl Operand for Liter8 {
  fn cycles() -> Cycles { Cycles(3) }
  fn size() -> u16 { 1 }
}

impl Src8 for Liter8 {
  fn load<C: Context>(&self, ctx: &C) -> u8 {
    ctx.read_op8()
  }
}


// A 16-bit literal operand
pub struct Liter16;

impl Operand for Liter16 {
  fn cycles() -> Cycles { Cycles(4) }
  fn size() -> u16 { 2 }
}

impl Src16 for Liter16 {
  fn load<C: Context>(&self, ctx: &C) -> u16 {
    ctx.read_op16()
  }
}


// A 8-bit direct address operand
pub struct Addr8;

impl Operand for Addr8 {
  fn cycles() -> Cycles { Cycles(7) }
  fn size() -> u16 { 2 }
}

impl Src8 for Addr8 {
  fn load<C: Context>(&self, ctx: &C) -> u8 {
    let addr = ctx.read_op16();
    ctx.mem().mem_read(addr)
  }
}

impl Dst8 for Addr8 {
  fn store<C: Context>(&self, ctx: &mut C, val: u8) {
    let addr = ctx.read_op16();
    ctx.mem_mut().mem_write(addr, val);
  }
}


// A 16-bit direct address operand
pub struct Addr16;

impl Operand for Addr16 {
  fn cycles() -> Cycles { Cycles(10) }
  fn size() -> u16 { 2 }
}

impl Src16 for Addr16 {
  fn load<C: Context>(&self, ctx: &C) -> u16 {
    let addr = ctx.read_op16();
    ctx.mem().mem_read16(addr)
  }
}

impl Dst16 for Addr16 {
  fn store<C: Context>(&self, ctx: &mut C, val: u16) {
    let addr = ctx.read_op16();
    ctx.mem_mut().mem_write16(addr, val);
  }
}



#[cfg(test)]
mod test {
  use super::*;

  use crate::cpu::z80::TestBench;

  #[test]
  fn reg8_load() {
    let mut ctx = TestBench::new();
    *ctx.regs.af.r16_mut() = 0x0100;
    *ctx.regs.bc.r16_mut() = 0x0203;
    *ctx.regs.de.r16_mut() = 0x0405;
    *ctx.regs.hl.r16_mut() = 0x0607;

    assert_eq!(0x01, Reg8::A.load(&ctx));
    assert_eq!(0x02, Reg8::B.load(&ctx));
    assert_eq!(0x03, Reg8::C.load(&ctx));
    assert_eq!(0x04, Reg8::D.load(&ctx));
    assert_eq!(0x05, Reg8::E.load(&ctx));
    assert_eq!(0x06, Reg8::H.load(&ctx));
    assert_eq!(0x07, Reg8::L.load(&ctx));
  }

  #[test]
  fn reg8_store() {
    let mut ctx = TestBench::new();
    Reg8::A.store(&mut ctx, 0x01);
    Reg8::B.store(&mut ctx, 0x02);
    Reg8::C.store(&mut ctx, 0x03);
    Reg8::D.store(&mut ctx, 0x04);
    Reg8::E.store(&mut ctx, 0x05);
    Reg8::H.store(&mut ctx, 0x06);
    Reg8::L.store(&mut ctx, 0x07);

    assert_eq!(0x01, ctx.regs.af.r8().h);
    assert_eq!(0x02, ctx.regs.bc.r8().h);
    assert_eq!(0x03, ctx.regs.bc.r8().l);
    assert_eq!(0x04, ctx.regs.de.r8().h);
    assert_eq!(0x05, ctx.regs.de.r8().l);
    assert_eq!(0x06, ctx.regs.hl.r8().h);
    assert_eq!(0x07, ctx.regs.hl.r8().l);
  }

  #[test]
  fn indreg8_load() {
    let mut ctx = TestBench::new();
    *ctx.regs.bc.r16_mut() = 0x1001;
    *ctx.regs.de.r16_mut() = 0x1002;
    *ctx.regs.hl.r16_mut() = 0x1003;
    ctx.mem_mut().mem_write(0x1001, 101);
    ctx.mem_mut().mem_write(0x1002, 102);
    ctx.mem_mut().mem_write(0x1003, 103);

    assert_eq!(101, IndReg8::BC.load(&ctx));
    assert_eq!(102, IndReg8::DE.load(&ctx));
    assert_eq!(103, IndReg8::HL.load(&ctx));
  }

  #[test]
  fn indreg8_store() {
    let mut ctx = TestBench::new();
    *ctx.regs.bc.r16_mut() = 0x1001;
    *ctx.regs.de.r16_mut() = 0x1002;
    *ctx.regs.hl.r16_mut() = 0x1003;
    IndReg8::BC.store(&mut ctx, 101);
    IndReg8::DE.store(&mut ctx, 102);
    IndReg8::HL.store(&mut ctx, 103);

    assert_eq!(101, ctx.mem().mem_read(0x1001));
    assert_eq!(102, ctx.mem().mem_read(0x1002));
    assert_eq!(103, ctx.mem().mem_read(0x1003));
  }

  #[test]
  fn reg16_load() {
    let mut ctx = TestBench::new();
    *ctx.regs.bc.r16_mut() = 1001;
    *ctx.regs.de.r16_mut() = 1002;
    *ctx.regs.hl.r16_mut() = 1003;

    assert_eq!(1001, Reg16::BC.load(&ctx));
    assert_eq!(1002, Reg16::DE.load(&ctx));
    assert_eq!(1003, Reg16::HL.load(&ctx));
  }

  #[test]
  fn reg16_store() {
    let mut ctx = TestBench::new();
    Reg16::BC.store(&mut ctx, 1001);
    Reg16::DE.store(&mut ctx, 1002);
    Reg16::HL.store(&mut ctx, 1003);

    assert_eq!(1001, ctx.regs().bc.r16());
    assert_eq!(1002, ctx.regs().de.r16());
    assert_eq!(1003, ctx.regs().hl.r16());
  }

  #[test]
  fn liter8_load() {
    let mut ctx = TestBench::new();
    ctx.regs_mut().pc = 0x4000;
    ctx.mem_mut().mem_write(0x4001, 101);
    assert_eq!(101, Liter8.load(&ctx));
  }

  #[test]
  fn liter16_load() {
    let mut ctx = TestBench::new();
    ctx.regs_mut().pc = 0x4000;
    ctx.mem_mut().mem_write16(0x4001, 0x1234);
    assert_eq!(0x1234, Liter16.load(&ctx));
  }

  #[test]
  fn addr8_load() {
    let mut ctx = TestBench::new();
    ctx.regs_mut().pc = 0x4000;
    ctx.mem_mut().mem_write16(0x4001, 0x1234);
    ctx.mem_mut().mem_write(0x1234, 101);
    assert_eq!(101, Addr8.load(& ctx));
  }

  #[test]
  fn addr8_store() {
    let mut ctx = TestBench::new();
    ctx.regs_mut().pc = 0x4000;
    ctx.mem_mut().mem_write16(0x4001, 0x1234);

    Addr8.store(&mut ctx, 101);

    assert_eq!(101, ctx.mem().mem_read(0x1234));
}

  #[test]
  fn addr16_load() {
    let mut ctx = TestBench::new();
    ctx.regs_mut().pc = 0x4000;
    ctx.mem_mut().mem_write16(0x4001, 0x1234);
    ctx.mem_mut().mem_write16(0x1234, 0x4567);
    assert_eq!(0x4567, Addr16.load(&ctx));
  }

  #[test]
  fn addr16_store() {
    let mut ctx = TestBench::new();
    ctx.regs_mut().pc = 0x4000;
    ctx.mem_mut().mem_write(0x4001, 0x34);
    ctx.mem_mut().mem_write(0x4002, 0x12);

    Addr16.store(&mut ctx, 0x4567);

    assert_eq!(0x4567, ctx.mem().mem_read16(0x1234));
  }
}
