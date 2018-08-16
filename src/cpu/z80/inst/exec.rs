use bus::{Address};
use cpu::z80::inst::ops::*;
use cpu::z80::inst::*;
use cpu::z80::reg;
use cpu::z80::reg::{FlagUpdate, Read, Write};

pub fn execute<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
    match inst {
        Inst{mnemo: Mnemo::ADD, ops: Operands::Binary8(dst, src), .. } => exec_add8(inst, ctx, dst, src),
        Inst{mnemo: Mnemo::ADD, ops: Operands::Binary16(dst, src), .. } => exec_add16(inst, ctx, dst, src),
        Inst{mnemo: Mnemo::DEC, ops: Operands::UnaryDest8(dst), .. } => exec_dec8(inst, ctx, dst),
        Inst{mnemo: Mnemo::DEC, ops: Operands::UnaryDest16(dst), .. } => exec_dec16(inst, ctx, dst),
        Inst{mnemo: Mnemo::EX, ops: Operands::UnaryDest8(_), .. } => exec_exaf(inst, ctx),
        Inst{mnemo: Mnemo::INC, ops: Operands::UnaryDest8(dst), .. } => exec_inc8(inst, ctx, dst),
        Inst{mnemo: Mnemo::INC, ops: Operands::UnaryDest16(dst), .. } => exec_inc16(inst, ctx, dst),
        Inst{mnemo: Mnemo::JP, ops: Operands::UnarySrc16(src), .. } => exec_jp(inst, ctx, src),
        Inst{mnemo: Mnemo::LD, ops: Operands::Binary16(dst, src), .. } => exec_load(inst, ctx, dst, src),
        Inst{mnemo: Mnemo::NOP, .. } => exec_nop(inst, ctx),
        Inst{mnemo: Mnemo::RLCA, .. } => exec_rlca(inst, ctx),
        Inst{mnemo: Mnemo::RRCA, .. } => exec_rrca(inst, ctx),
        _ => unimplemented!("cannot execute illegal instruction"),
    }    
}

fn exec_add8<C: Context>(inst: &Inst, ctx: &mut C, dst: &Dest8, src: &Src8) -> Cycles {
    let a = src.read(ctx);
    let b = dst.read(ctx);
    let c = (a as u16) + (b as u16);
    dst.write(ctx, c as u8);
    ctx.regs_mut().inc_pc(inst.size);
    let flags = FlagUpdate::new()
        .C(c > 0xff)
        .H(c > 0xff)
        .N(false)
        .PV(c > 0xff)
        .S(c > 0xff)
        .Z(c == 0);
    ctx.regs_mut().update_flags(flags);
    inst.cycles
}

fn exec_add16<C: Context>(inst: &Inst, ctx: &mut C, dst: &Dest16, src: &Src16) -> Cycles {
    let a = src.read(ctx);
    let b = dst.read(ctx);
    let c = (a as u32) + (b as u32);
    dst.write(ctx, c as u16);
    ctx.regs_mut().inc_pc(inst.size);
    let flags = FlagUpdate::new()
        .C(c > 0xffff)
        .N(true)
        .H(c > 0x00ff);
    ctx.regs_mut().update_flags(flags);
    inst.cycles
}

fn exec_nop<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
    ctx.regs_mut().inc_pc(inst.size);
    inst.cycles
}

fn exec_inc8<C: Context>(inst: &Inst, ctx: &mut C, dst: &Dest8) -> Cycles {
    let val = dst.read(ctx);
    dst.write(ctx, val + 1);
    ctx.regs_mut().inc_pc(inst.size);
    let flags = FlagUpdate::with_opcode(inst.opcode).N(true).PV(val == 0xff);
    ctx.regs_mut().update_flags(flags);
    inst.cycles
}

fn exec_inc16<C: Context>(inst: &Inst, ctx: &mut C, dst: &Dest16) -> Cycles {
    let val = dst.read(ctx);
    dst.write(ctx, val + 1);
    ctx.regs_mut().inc_pc(inst.size);
    inst.cycles
}

fn exec_jp<C: Context>(inst: &Inst, ctx: &mut C, src: &Src16) -> Cycles {
    let val = src.read(ctx);
    ctx.regs_mut().set_pc(Address::from(val));
    inst.cycles
}

fn exec_dec8<C: Context>(inst: &Inst, ctx: &mut C, dst: &Dest8) -> Cycles {
    let val = dst.read(ctx);
    dst.write(ctx, val - 1);
    ctx.regs_mut().inc_pc(inst.size);
    let flags = FlagUpdate::with_opcode(inst.opcode).N(false).PV(val == 0);
    ctx.regs_mut().update_flags(flags);
    inst.cycles
}

fn exec_dec16<C: Context>(inst: &Inst, ctx: &mut C, dst: &Dest16) -> Cycles {
    let val = dst.read(ctx);
    dst.write(ctx, val - 1);
    ctx.regs_mut().inc_pc(inst.size);
    inst.cycles
}

fn exec_exaf<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
    ctx.regs_mut().swap_af();
    ctx.regs_mut().inc_pc(inst.size);
    inst.cycles
}

fn exec_load<C: Context, D: Data>(inst: &Inst, ctx: &mut C, dst: &Dest<D>, src: &Src<D>) -> Cycles {
    let val = src.read(ctx);
    dst.write(ctx, val);
    ctx.regs_mut().inc_pc(inst.size);
    inst.cycles
}

fn exec_rlca<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
    let orig = reg::Name8::A.read(ctx.regs());
    let carry = orig >> 7;
    let dest = (orig << 1) | carry;
    reg::Name8::A.write(ctx.regs_mut(), dest);
    ctx.regs_mut().inc_pc(inst.size);
    let flags = FlagUpdate::with_opcode(inst.opcode)
        .C(carry > 0)
        .H(false)
        .N(false);
    ctx.regs_mut().update_flags(flags);

    inst.cycles
}

fn exec_rrca<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
    let orig = reg::Name8::A.read(ctx.regs());
    let carry = orig << 7;
    let dest = (orig >> 1) | carry;
    reg::Name8::A.write(ctx.regs_mut(), dest);
    ctx.regs_mut().inc_pc(inst.size);
    let flags = FlagUpdate::with_opcode(inst.opcode)
        .C(carry > 0)
        .H(false)
        .N(false);
    ctx.regs_mut().update_flags(flags);
    inst.cycles
}


