use std::num::Wrapping;

use crate::cpu::z80::{Cycles, MemBus, IOBus};
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
  fn cycles() -> Cycles { 0 }
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
  fn cycles() -> Cycles { 3 }
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
  fn cycles() -> Cycles { 0 }
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


// A 16-bit literal operand
pub struct Liter16;

impl Operand for Liter16 {
  fn cycles() -> Cycles { 4 }
  fn size() -> u16 { 2 }
}

impl Src16 for Liter16 {
  fn load<C: Context>(&self, ctx: &C) -> u16 {
    ctx.read_op16()
  }
}



#[cfg(test)]
mod test {
  use super::*;

  use crate::cpu::z80::CPU;

  #[test]
  fn reg8_load() {
      let mut cpu = CPU::testbench();
      *cpu.regs.af.r16_mut() = 0x0100;
      *cpu.regs.bc.r16_mut() = 0x0203;
      *cpu.regs.de.r16_mut() = 0x0405;
      *cpu.regs.hl.r16_mut() = 0x0607;

      assert_eq!(0x01, Reg8::A.load(&cpu));
      assert_eq!(0x02, Reg8::B.load(&cpu));
      assert_eq!(0x03, Reg8::C.load(&cpu));
      assert_eq!(0x04, Reg8::D.load(&cpu));
      assert_eq!(0x05, Reg8::E.load(&cpu));
      assert_eq!(0x06, Reg8::H.load(&cpu));
      assert_eq!(0x07, Reg8::L.load(&cpu));
  }

  #[test]
  fn reg8_store() {
      let mut cpu = CPU::testbench();
      Reg8::A.store(&mut cpu, 0x01);
      Reg8::B.store(&mut cpu, 0x02);
      Reg8::C.store(&mut cpu, 0x03);
      Reg8::D.store(&mut cpu, 0x04);
      Reg8::E.store(&mut cpu, 0x05);
      Reg8::H.store(&mut cpu, 0x06);
      Reg8::L.store(&mut cpu, 0x07);

      assert_eq!(0x01, cpu.regs.af.r8().h);
      assert_eq!(0x02, cpu.regs.bc.r8().h);
      assert_eq!(0x03, cpu.regs.bc.r8().l);
      assert_eq!(0x04, cpu.regs.de.r8().h);
      assert_eq!(0x05, cpu.regs.de.r8().l);
      assert_eq!(0x06, cpu.regs.hl.r8().h);
      assert_eq!(0x07, cpu.regs.hl.r8().l);
  }

  #[test]
  fn indreg8_load() {
      let mut cpu = CPU::testbench();
      *cpu.regs.bc.r16_mut() = 0x1001;
      *cpu.regs.de.r16_mut() = 0x1002;
      *cpu.regs.hl.r16_mut() = 0x1003;
      cpu.mem_mut().mem_write(0x1001, 101);
      cpu.mem_mut().mem_write(0x1002, 102);
      cpu.mem_mut().mem_write(0x1003, 103);

      assert_eq!(101, IndReg8::BC.load(&cpu));
      assert_eq!(102, IndReg8::DE.load(&cpu));
      assert_eq!(103, IndReg8::HL.load(&cpu));
  }

  #[test]
  fn indreg8_store() {
      let mut cpu = CPU::testbench();
      *cpu.regs.bc.r16_mut() = 0x1001;
      *cpu.regs.de.r16_mut() = 0x1002;
      *cpu.regs.hl.r16_mut() = 0x1003;
      IndReg8::BC.store(&mut cpu, 101);
      IndReg8::DE.store(&mut cpu, 102);
      IndReg8::HL.store(&mut cpu, 103);

      assert_eq!(101, cpu.mem().mem_read(0x1001));
      assert_eq!(102, cpu.mem().mem_read(0x1002));
      assert_eq!(103, cpu.mem().mem_read(0x1003));
  }

  #[test]
  fn reg16_load() {
      let mut cpu = CPU::testbench();
      *cpu.regs.bc.r16_mut() = 1001;
      *cpu.regs.de.r16_mut() = 1002;
      *cpu.regs.hl.r16_mut() = 1003;

      assert_eq!(1001, Reg16::BC.load(&cpu));
      assert_eq!(1002, Reg16::DE.load(&cpu));
      assert_eq!(1003, Reg16::HL.load(&cpu));
  }

  #[test]
  fn reg16_store() {
      let mut cpu = CPU::testbench();
      Reg16::BC.store(&mut cpu, 1001);
      Reg16::DE.store(&mut cpu, 1002);
      Reg16::HL.store(&mut cpu, 1003);

      assert_eq!(1001, cpu.regs().bc.r16());
      assert_eq!(1002, cpu.regs().de.r16());
      assert_eq!(1003, cpu.regs().hl.r16());
  }

  #[test]
  fn liter16_load() {
      let mut cpu = CPU::testbench();
      cpu.regs_mut().pc = 0x4000;
      cpu.mem_mut().mem_write(0x4001, 0x34);
      cpu.mem_mut().mem_write(0x4002, 0x12);
      assert_eq!(0x1234, Liter16.load(& cpu));
  }
}
