use bus::{Address};
use cpu::z80::data::Data;
use cpu::z80::inst::ops::*;
use cpu::z80::inst::*;
use cpu::z80::regs::{Reg8, Register};

pub fn execute<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
    match inst {
        Inst{mnemo: Mnemo::ADD, ops: Operands::Binary8(dst, src), .. } => exec_add(inst, ctx, dst, src),
        Inst{mnemo: Mnemo::ADD, ops: Operands::Binary16(dst, src), .. } => exec_add(inst, ctx, dst, src),
        Inst{mnemo: Mnemo::DEC, ops: Operands::UnaryDest8(dst), .. } => exec_dec(inst, ctx, dst),
        Inst{mnemo: Mnemo::DEC, ops: Operands::UnaryDest16(dst), .. } => exec_dec(inst, ctx, dst),
        Inst{mnemo: Mnemo::EX, ops: Operands::UnaryDest8(_), .. } => exec_exaf(inst, ctx),
        Inst{mnemo: Mnemo::INC, ops: Operands::UnaryDest8(dst), .. } => exec_inc(inst, ctx, dst),
        Inst{mnemo: Mnemo::INC, ops: Operands::UnaryDest16(dst), .. } => exec_inc(inst, ctx, dst),
        Inst{mnemo: Mnemo::JP, ops: Operands::UnarySrc16(src), .. } => exec_jp(inst, ctx, src),
        Inst{mnemo: Mnemo::LD, ops: Operands::Binary16(dst, src), .. } => exec_load(inst, ctx, dst, src),
        Inst{mnemo: Mnemo::NOP, .. } => exec_nop(inst, ctx),
        Inst{mnemo: Mnemo::RLCA, .. } => exec_rlca(inst, ctx),
        Inst{mnemo: Mnemo::RRCA, .. } => exec_rrca(inst, ctx),
        _ => unimplemented!("cannot execute illegal instruction"),
    }    
}

fn exec_add<C: Context, D: Data>(inst: &Inst, ctx: &mut C, dst: &Dest<D>, src: &Src<D>) -> Cycles {
    let a = src.read(ctx);
    let b = dst.read(ctx);
    dst.write(ctx, a + b);
    ctx.regs_mut().inc_pc(inst.size);
    inst.cycles
}

fn exec_nop<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
    ctx.regs_mut().inc_pc(inst.size);
    inst.cycles
}

fn exec_inc<C: Context, D: Data>(inst: &Inst, ctx: &mut C, dst: &Dest<D>) -> Cycles {
    let val = dst.read(ctx);
    dst.write(ctx, D::inc(val));
    ctx.regs_mut().inc_pc(inst.size);
    inst.cycles
}

fn exec_jp<C: Context>(inst: &Inst, ctx: &mut C, src: &Src16) -> Cycles {
    let val = src.read(ctx);
    ctx.regs_mut().set_pc(Address::from(val));
    inst.cycles
}

fn exec_dec<C: Context, D: Data>(inst: &Inst, ctx: &mut C, dst: &Dest<D>) -> Cycles {
    let val = dst.read(ctx);
    dst.write(ctx, D::dec(val));
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
    let orig = Reg8::A.read(ctx.regs());
    let dest = (orig << 1) | (orig >> 7);
    Reg8::A.write(ctx.regs_mut(), dest);
    inst.cycles
}

fn exec_rrca<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
    let orig = Reg8::A.read(ctx.regs());
    let dest = (orig >> 1) | (orig << 7);
    Reg8::A.write(ctx.regs_mut(), dest);
    inst.cycles
}


