use std::num::Wrapping;

use crate::emu::Cycles;
use crate::cpu::z80::MemBus;
use crate::cpu::z80::exec::*;

pub fn exec_inst<C: Context>(ctx: &mut C) -> Cycles {
  let pc = ctx.regs().pc_addr();
  let opcode = ctx.mem().mem_read(pc);
  match opcode {
    0x00 => nop(ctx),
    0x01 => ld16(ctx, Reg16::BC, Liter16),
    0x02 => ld8(ctx, IndReg8::BC, Reg8::A),
    0x03 => unimplemented!(),
    0x04 => unimplemented!(),
    0x05 => unimplemented!(),
    0x06 => ld8(ctx, Reg8::B, Liter8),
    0x07 => unimplemented!(),
    0x08 => unimplemented!(),
    0x09 => unimplemented!(),
    0x0a => ld8(ctx, Reg8::A, IndReg8::BC),
    0x0b => unimplemented!(),
    0x0c => unimplemented!(),
    0x0d => unimplemented!(),
    0x0e => ld8(ctx, Reg8::C, Liter8),
    0x0f => unimplemented!(),

    0x10 => unimplemented!(),
    0x11 => ld16(ctx, Reg16::DE, Liter16),
    0x12 => ld8(ctx, IndReg8::DE, Reg8::A),
    0x13 => unimplemented!(),
    0x14 => unimplemented!(),
    0x15 => unimplemented!(),
    0x16 => ld8(ctx, Reg8::D, Liter8),
    0x17 => unimplemented!(),
    0x18 => unimplemented!(),
    0x19 => unimplemented!(),
    0x1a => ld8(ctx, Reg8::A, IndReg8::DE),
    0x1b => unimplemented!(),
    0x1c => unimplemented!(),
    0x1d => unimplemented!(),
    0x1e => ld8(ctx, Reg8::E, Liter8),
    0x1f => unimplemented!(),

    0x20 => unimplemented!(),
    0x21 => ld16(ctx, Reg16::HL, Liter16),
    0x22 => ld16(ctx, Addr16, Reg16::HL),
    0x23 => unimplemented!(),
    0x24 => unimplemented!(),
    0x25 => unimplemented!(),
    0x26 => ld8(ctx, Reg8::H, Liter8),
    0x27 => unimplemented!(),
    0x28 => unimplemented!(),
    0x29 => unimplemented!(),
    0x2a => ld16(ctx, Reg16::HL, Addr16),
    0x2b => unimplemented!(),
    0x2c => unimplemented!(),
    0x2d => unimplemented!(),
    0x2e => ld8(ctx, Reg8::L, Liter8),
    0x2f => unimplemented!(),

    0x30 => unimplemented!(),
    0x31 => ld16(ctx, Reg16::SP, Liter16),
    0x32 => ld8(ctx, Addr8, Reg8::A),
    0x33 => unimplemented!(),
    0x34 => unimplemented!(),
    0x35 => unimplemented!(),
    0x36 => ld8(ctx, IndReg8::HL, Liter8),
    0x37 => unimplemented!(),
    0x38 => unimplemented!(),
    0x39 => unimplemented!(),
    0x3a => ld8(ctx, Reg8::A, Addr8),
    0x3b => unimplemented!(),
    0x3c => unimplemented!(),
    0x3d => unimplemented!(),
    0x3e => ld8(ctx, Reg8::A, Liter8),
    0x3f => unimplemented!(),

    0x40 => ld8(ctx, Reg8::B, Reg8::B),
    0x41 => ld8(ctx, Reg8::B, Reg8::C),
    0x42 => ld8(ctx, Reg8::B, Reg8::D),
    0x43 => ld8(ctx, Reg8::B, Reg8::E),
    0x44 => ld8(ctx, Reg8::B, Reg8::H),
    0x45 => ld8(ctx, Reg8::B, Reg8::L),
    0x46 => ld8(ctx, Reg8::B, IndReg8::HL),
    0x47 => ld8(ctx, Reg8::B, Reg8::A),
    0x48 => ld8(ctx, Reg8::C, Reg8::B),
    0x49 => ld8(ctx, Reg8::C, Reg8::C),
    0x4a => ld8(ctx, Reg8::C, Reg8::D),
    0x4b => ld8(ctx, Reg8::C, Reg8::E),
    0x4c => ld8(ctx, Reg8::C, Reg8::H),
    0x4d => ld8(ctx, Reg8::C, Reg8::L),
    0x4e => ld8(ctx, Reg8::C, IndReg8::HL),
    0x4f => ld8(ctx, Reg8::C, Reg8::A),

    0x50 => ld8(ctx, Reg8::D, Reg8::B),
    0x51 => ld8(ctx, Reg8::D, Reg8::C),
    0x52 => ld8(ctx, Reg8::D, Reg8::D),
    0x53 => ld8(ctx, Reg8::D, Reg8::E),
    0x54 => ld8(ctx, Reg8::D, Reg8::H),
    0x55 => ld8(ctx, Reg8::D, Reg8::L),
    0x56 => ld8(ctx, Reg8::D, IndReg8::HL),
    0x57 => ld8(ctx, Reg8::D, Reg8::A),
    0x58 => ld8(ctx, Reg8::E, Reg8::B),
    0x59 => ld8(ctx, Reg8::E, Reg8::C),
    0x5a => ld8(ctx, Reg8::E, Reg8::D),
    0x5b => ld8(ctx, Reg8::E, Reg8::E),
    0x5c => ld8(ctx, Reg8::E, Reg8::H),
    0x5d => ld8(ctx, Reg8::E, Reg8::L),
    0x5e => ld8(ctx, Reg8::E, IndReg8::HL),
    0x5f => ld8(ctx, Reg8::E, Reg8::A),

    0x60 => ld8(ctx, Reg8::H, Reg8::B),
    0x61 => ld8(ctx, Reg8::H, Reg8::C),
    0x62 => ld8(ctx, Reg8::H, Reg8::D),
    0x63 => ld8(ctx, Reg8::H, Reg8::E),
    0x64 => ld8(ctx, Reg8::H, Reg8::H),
    0x65 => ld8(ctx, Reg8::H, Reg8::L),
    0x66 => ld8(ctx, Reg8::H, IndReg8::HL),
    0x67 => ld8(ctx, Reg8::H, Reg8::A),
    0x68 => ld8(ctx, Reg8::L, Reg8::B),
    0x69 => ld8(ctx, Reg8::L, Reg8::C),
    0x6a => ld8(ctx, Reg8::L, Reg8::D),
    0x6b => ld8(ctx, Reg8::L, Reg8::E),
    0x6c => ld8(ctx, Reg8::L, Reg8::H),
    0x6d => ld8(ctx, Reg8::L, Reg8::L),
    0x6e => ld8(ctx, Reg8::L, IndReg8::HL),
    0x6f => ld8(ctx, Reg8::L, Reg8::A),

    0x70 => ld8(ctx, IndReg8::HL, Reg8::B),
    0x71 => ld8(ctx, IndReg8::HL, Reg8::C),
    0x72 => ld8(ctx, IndReg8::HL, Reg8::D),
    0x73 => ld8(ctx, IndReg8::HL, Reg8::E),
    0x74 => ld8(ctx, IndReg8::HL, Reg8::H),
    0x75 => ld8(ctx, IndReg8::HL, Reg8::L),
    0x76 => unimplemented!(),
    0x77 => ld8(ctx, IndReg8::HL, Reg8::A),
    0x78 => ld8(ctx, Reg8::A, Reg8::B),
    0x79 => ld8(ctx, Reg8::A, Reg8::C),
    0x7a => ld8(ctx, Reg8::A, Reg8::D),
    0x7b => ld8(ctx, Reg8::A, Reg8::E),
    0x7c => ld8(ctx, Reg8::A, Reg8::H),
    0x7d => ld8(ctx, Reg8::A, Reg8::L),
    0x7e => ld8(ctx, Reg8::A, IndReg8::HL),
    0x7f => ld8(ctx, Reg8::A, Reg8::A),

    0x80 => add8(ctx, Reg8::B),
    0x81 => add8(ctx, Reg8::C),
    0x82 => add8(ctx, Reg8::D),
    0x83 => add8(ctx, Reg8::H),
    0x84 => add8(ctx, Reg8::L),
    0x85 => add8(ctx, IndReg8::HL),
    0x86 => add8(ctx, Reg8::A),
    0x87 => unimplemented!(),
    0x88 => unimplemented!(),
    0x89 => unimplemented!(),
    0x8a => unimplemented!(),
    0x8b => unimplemented!(),
    0x8c => unimplemented!(),
    0x8d => unimplemented!(),
    0x8e => unimplemented!(),
    0x8f => unimplemented!(),

    0x90 => unimplemented!(),
    0x91 => unimplemented!(),
    0x92 => unimplemented!(),
    0x93 => unimplemented!(),
    0x94 => unimplemented!(),
    0x95 => unimplemented!(),
    0x96 => unimplemented!(),
    0x97 => unimplemented!(),
    0x98 => unimplemented!(),
    0x99 => unimplemented!(),
    0x9a => unimplemented!(),
    0x9b => unimplemented!(),
    0x9c => unimplemented!(),
    0x9d => unimplemented!(),
    0x9e => unimplemented!(),
    0x9f => unimplemented!(),

    0xa0 => unimplemented!(),
    0xa1 => unimplemented!(),
    0xa2 => unimplemented!(),
    0xa3 => unimplemented!(),
    0xa4 => unimplemented!(),
    0xa5 => unimplemented!(),
    0xa6 => unimplemented!(),
    0xa7 => unimplemented!(),
    0xa8 => unimplemented!(),
    0xa9 => unimplemented!(),
    0xaa => unimplemented!(),
    0xab => unimplemented!(),
    0xac => unimplemented!(),
    0xad => unimplemented!(),
    0xae => unimplemented!(),
    0xaf => unimplemented!(),

    0xb0 => unimplemented!(),
    0xb1 => unimplemented!(),
    0xb2 => unimplemented!(),
    0xb3 => unimplemented!(),
    0xb4 => unimplemented!(),
    0xb5 => unimplemented!(),
    0xb6 => unimplemented!(),
    0xb7 => unimplemented!(),
    0xb8 => unimplemented!(),
    0xb9 => unimplemented!(),
    0xba => unimplemented!(),
    0xbb => unimplemented!(),
    0xbc => unimplemented!(),
    0xbd => unimplemented!(),
    0xbe => unimplemented!(),
    0xbf => unimplemented!(),

    0xc0 => unimplemented!(),
    0xc1 => unimplemented!(),
    0xc2 => unimplemented!(),
    0xc3 => unimplemented!(),
    0xc4 => unimplemented!(),
    0xc5 => unimplemented!(),
    0xc6 => unimplemented!(),
    0xc7 => unimplemented!(),
    0xc8 => unimplemented!(),
    0xc9 => unimplemented!(),
    0xca => unimplemented!(),
    0xcb => unimplemented!(),
    0xcc => unimplemented!(),
    0xcd => unimplemented!(),
    0xce => unimplemented!(),
    0xcf => unimplemented!(),

    0xd0 => unimplemented!(),
    0xd1 => unimplemented!(),
    0xd2 => unimplemented!(),
    0xd3 => unimplemented!(),
    0xd4 => unimplemented!(),
    0xd5 => unimplemented!(),
    0xd6 => unimplemented!(),
    0xd7 => unimplemented!(),
    0xd8 => unimplemented!(),
    0xd9 => unimplemented!(),
    0xda => unimplemented!(),
    0xdb => unimplemented!(),
    0xdc => unimplemented!(),
    0xdd => unimplemented!(),
    0xde => unimplemented!(),
    0xdf => unimplemented!(),

    0xe0 => unimplemented!(),
    0xe1 => unimplemented!(),
    0xe2 => unimplemented!(),
    0xe3 => unimplemented!(),
    0xe4 => unimplemented!(),
    0xe5 => unimplemented!(),
    0xe6 => unimplemented!(),
    0xe7 => unimplemented!(),
    0xe8 => unimplemented!(),
    0xe9 => unimplemented!(),
    0xea => unimplemented!(),
    0xeb => unimplemented!(),
    0xec => unimplemented!(),
    0xed => unimplemented!(),
    0xee => unimplemented!(),
    0xef => unimplemented!(),

    0xf0 => unimplemented!(),
    0xf1 => unimplemented!(),
    0xf2 => unimplemented!(),
    0xf3 => unimplemented!(),
    0xf4 => unimplemented!(),
    0xf5 => unimplemented!(),
    0xf6 => unimplemented!(),
    0xf7 => unimplemented!(),
    0xf8 => unimplemented!(),
    0xf9 => ld16(ctx, Reg16::SP, Reg16::HL),
    0xfa => unimplemented!(),
    0xfb => unimplemented!(),
    0xfc => unimplemented!(),
    0xfd => unimplemented!(),
    0xfe => unimplemented!(),
    0xff => unimplemented!(),
  }
}

pub fn nop<C: Context>(ctx: &mut C) -> Cycles {
  ctx.regs_mut().inc_pc(1);
  Cycles(4)
}

pub fn add8<C: Context, S: Src8>(ctx: &mut C, src: S) -> Cycles {
  let a = ctx.regs().af.r8().h;
  let b = src.load(ctx);
  let c = (Wrapping(a) + Wrapping(b)).0;
  ctx.regs_mut().af.r8_mut().h = c;
  ctx.regs_mut().inc_pc(1 + S::size());
  Cycles(4) + S::cycles()
}

pub fn ld8<C: Context, S: Src8, D: Dst8>(ctx: &mut C, dst: D, src: S) -> Cycles {
  let v = src.load(ctx);
  dst.store(ctx, v);
  ctx.regs_mut().inc_pc(1 + S::size() + D::size());
  Cycles(4) + S::cycles() + D::cycles()
}

pub fn ld16<C: Context, S: Src16, D: Dst16>(ctx: &mut C, dst: D, src: S) -> Cycles {
  let v = src.load(ctx);
  dst.store(ctx, v);
  ctx.regs_mut().inc_pc(1 + S::size() + D::size());
  Cycles(6) + S::cycles() + D::cycles()
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::cpu::z80::{MemAddr, TestBench};

  #[test]
  fn inst_nop() {
    let mut ctx = TestBench::new();
    let cycles = nop(&mut ctx);
    assert_eq!(0x0001, ctx.regs.pc);
    assert_eq!(Cycles(4), cycles);
  }

  #[test]
  fn inst_add8() {
    let mut ctx = TestBench::new();
    ctx.regs.af.r8_mut().h = 0x42;
    *ctx.regs.hl.r16_mut() = 0x4001;
    ctx.mem_mut().mem_write(MemAddr(0x4001), 0x19);
    let cycles = add8(&mut ctx, IndReg8::HL);
    assert_eq!(0x5b, ctx.regs.af.r8().h);
    assert_eq!(0x0001, ctx.regs.pc);
    assert_eq!(Cycles(7), cycles);
  }

  #[test]
  fn inst_ld8() {
    let mut ctx = TestBench::new();
    ctx.regs.bc.r8_mut().h = 0x42;
    let cycles = ld8(&mut ctx, Reg8::A, Reg8::B);
    assert_eq!(0x42, ctx.regs.af.r8().h);
    assert_eq!(0x0001, ctx.regs.pc);
    assert_eq!(Cycles(4), cycles);
  }

  #[test]
  fn inst_ld16() {
    let mut ctx = TestBench::new();
    *ctx.regs_mut().hl.r16_mut() = 1001;
    let cycles = ld16(&mut ctx, Reg16::SP, Reg16::HL);
    assert_eq!(1001, ctx.regs.sp);
    assert_eq!(0x0001, ctx.regs.pc);
    assert_eq!(Cycles(6), cycles);
  }
}
