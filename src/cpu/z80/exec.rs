use byteorder::LittleEndian;

use bus::{Bus, ReadFromBytes};
use cpu::z80::{Cycles, Inst, MemoryBus, Registers};

// Context trait defines a context where instructions are executed
pub trait Context {
    type Mem: MemoryBus;
    fn regs(&self) -> &Registers;
    fn regs_mut(&mut self) -> &mut Registers;
    fn mem(&self) -> &Self::Mem;
    fn mem_mut(&mut self) -> &mut Self::Mem;
}

pub fn execute<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
    match inst.opcode {
        0x00 => exec_nop(inst, ctx),
        0x01 => exec_ld_bc_l16(inst, ctx),
        0x02 => exec_ld_indbc_a(inst, ctx),
        0x03 => exec_inc_bc(inst, ctx),
        0x04 => exec_inc_b(inst, ctx),
        0x05 => exec_dec_b(inst, ctx),
        0x06 => exec_ld_b_l8(inst, ctx),
        0x07 => exec_rlca(inst, ctx),
        0x08 => exec_exaf(inst, ctx),
        0x09 => exec_add_hl_bc(inst, ctx),
        0x0a => exec_ld_a_indbc(inst, ctx),
        0x0b => exec_dec_bc(inst, ctx),
        0x0c => exec_inc_c(inst, ctx),
        0x0d => exec_dec_c(inst, ctx),
        0x0e => exec_ld_c_l8(inst, ctx),
        0x0f => exec_rrca(inst, ctx),
        _ => unimplemented!("cannot execute illegal instruction"),
    }       
}

pub fn exec_step<C: Context>(ctx: &mut C) -> Cycles {
    let pc = *ctx.regs().pc;
    let opcode = ctx.mem().read(pc);
    match opcode {
        0x00 => exec_nop(&inst!(NOP), ctx),
        0x01 => exec_ld_bc_l16(&inst!(LD BC, ctx.mem().read_word::<LittleEndian>(pc + 1)), ctx),
        0x02 => exec_ld_indbc_a(&inst!(LD (BC), A), ctx),
        0x03 => exec_inc_bc(&inst!(INC BC), ctx),
        0x04 => exec_inc_b(&inst!(INC B), ctx),
        0x05 => exec_dec_b(&inst!(DEC B), ctx),
        0x06 => exec_ld_b_l8(&inst!(LD B, ctx.mem().read(pc + 1)), ctx),
        0x07 => exec_rlca(&inst!(RLCA), ctx),
        0x08 => exec_exaf(&inst!(EX AF, AF_), ctx),
        0x09 => exec_add_hl_bc(&inst!(ADD HL, BC), ctx),
        0x0a => exec_ld_a_indbc(&inst!(LD A, (BC)), ctx),
        0x0b => exec_dec_bc(&inst!(DEC BC), ctx),
        0x0c => exec_inc_c(&inst!(INC C), ctx),
        0x0d => exec_dec_c(&inst!(DEC C), ctx),
        0x0e => exec_ld_c_l8(&inst!(LD C, ctx.mem().read(pc + 1)), ctx),
        0x0f => exec_rrca(&inst!(RRCA), ctx),
        0xc3 => exec_jp_l16(&inst!(JP ctx.mem().read_word::<LittleEndian>(pc + 1)), ctx),
        _ => unimplemented!("cannot execute illegal instruction with opcode 0x{:x}", opcode),
    }
}

macro_rules! read_arg {
    // 8-bits registers
    ($ctx:expr, $inst:expr, A) => (unsafe { $ctx.regs().af.as_byte.h });
    ($ctx:expr, $inst:expr, B) => (unsafe { $ctx.regs().bc.as_byte.h });
    ($ctx:expr, $inst:expr, C) => (unsafe { $ctx.regs().bc.as_byte.l });
    // 16-bits registers
    ($ctx:expr, $inst:expr, BC) => (*($ctx.regs().bc));
    ($ctx:expr, $inst:expr, HL) => (*($ctx.regs().hl));
    ($ctx:expr, $inst:expr, INDBC) => ($ctx.mem().read(*($ctx.regs().bc)));
    // literals
    ($ctx:expr, $inst:expr, L8) => ($inst.extra8);
    ($ctx:expr, $inst:expr, L16) => ($inst.extra16);
}

macro_rules! write_arg {
    // 8-bits registers
    ($ctx:expr, A, $val:expr) => (unsafe { $ctx.regs_mut().af.as_byte.h = $val });
    ($ctx:expr, B, $val:expr) => (unsafe { $ctx.regs_mut().bc.as_byte.h = $val });
    ($ctx:expr, C, $val:expr) => (unsafe { $ctx.regs_mut().bc.as_byte.l = $val });
    // 16-bits registers
    ($ctx:expr, HL, $val:expr) => (*($ctx.regs_mut().hl) = $val);
    ($ctx:expr, BC, $val:expr) => (*($ctx.regs_mut().bc) = $val);    
    ($ctx:expr, INDBC, $val:expr) => ({
        let addr = *($ctx.regs().bc);
        $ctx.mem_mut().write(addr, $val)
    });
}

macro_rules! flags_bitmask_set {
    (C)         => (0b00000001);
    (N)         => (0b00000010);
    (PV)        => (0b00000100);
    (H)         => (0b00010000);
    (Z)         => (0b01000000);
    (S)         => (0b10000000);
    ($($a:ident),+) => ($(flags_bitmask_reset!($a))|+);
}

macro_rules! flags_bitmask_reset {
    (C)         => (0b11111110);
    (N)         => (0b11111101);
    (PV)        => (0b11111011);
    (H)         => (0b11101111);
    (Z)         => (0b10111111);
    (S)         => (0b01111111);
    ($($a:ident),+) => ($(flags_bitmask_reset!($a))&+);
}

macro_rules! flags_apply {
    ($a:expr, ) => ($a);
    ($a:expr, $f:ident:0 $($rest:tt)*) => (flags_apply!($a & flags_bitmask_reset!($f), $($rest)*));
    ($a:expr, $f:ident:1 $($rest:tt)*) => (flags_apply!($a | flags_bitmask_set!($f), $($rest)*));
    ($a:expr, $f:ident:[$c:expr] $($rest:tt)*) => (flags_apply!(if $c { $a | flags_bitmask_set!($f) } else { $a & flags_bitmask_reset!($f) }, $($rest)*));
    ($a:expr, [$($f:ident),+]:[$c:expr] $($rest:tt)*) => (flags_apply!(if $c { $a | flags_bitmask_set!($($f),+) } else { $a & flags_bitmask_reset!($($f),+) }, $($rest)*));
}

macro_rules! exec_func {
    ($name:ident, ADD8, $dst:tt, $src:tt) => (
        fn $name<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
            let a = read_arg!(ctx, inst, $src);
            let b = read_arg!(ctx, inst, $dst);
            let c = (a as u16) + (b as u16);
            write_arg!(ctx, $dst, c as u8);
            ctx.regs_mut().inc_pc(inst.size);
            let flags = flags_apply!(ctx.regs().flags(), [C,H,PV,S]:[c > 0xff] Z:[c == 0] N:0)
            ctx.regs_mut().set_flags(flags);
            inst.cycles
        }
    );
    ($name:ident, ADD16, $dst:tt, $src:tt) => (
        fn $name<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
            let a = read_arg!(ctx, inst, $dst);
            let b = read_arg!(ctx, inst, $src);
            let c = (a as u32) + (b as u32);
            write_arg!(ctx, $dst, c as u16);
            ctx.regs_mut().inc_pc(inst.size);

            let flags = flags_apply!(ctx.regs().flags(), [C,H]:[c>0xffff] N:1);
            ctx.regs_mut().set_flags(flags);

            inst.cycles
        }
    );
    ($name:ident, INC8, $dst:tt) => (
        fn $name<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
            let val = read_arg!(ctx, inst, $dst);
            write_arg!(ctx, $dst, val + 1);
            ctx.regs_mut().inc_pc(inst.size);
            let flags = flags_apply!(ctx.regs().flags(), N:1 PV:[val == 0xff]);
            ctx.regs_mut().set_flags(flags);
            inst.cycles
        }
    );
    ($name:ident, INC16, $dst:tt) => (
        fn $name<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
            let val = read_arg!(ctx, inst, $dst);
            write_arg!(ctx, $dst, val + 1);
            ctx.regs_mut().inc_pc(inst.size);
            inst.cycles
        }
    );
    ($name:ident, DEC8, $dst:tt) => (
        fn $name<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
            let val = read_arg!(ctx, inst, $dst);
            write_arg!(ctx, $dst, val - 1);
            ctx.regs_mut().inc_pc(inst.size);
            let flags = flags_apply!(ctx.regs().flags(), N:0 PV:[val == 0]);
            ctx.regs_mut().set_flags(flags);
            inst.cycles
        }
    );
    ($name:ident, DEC16, $dst:tt) => (
        fn $name<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
            let val = read_arg!(ctx, inst, $dst);
            write_arg!(ctx, $dst, val - 1);
            ctx.regs_mut().inc_pc(inst.size);
            inst.cycles
        }
    );
    ($name:ident, JP, $src:tt) => (
        fn $name<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
            let val = read_arg!(ctx, inst, $src);
            *ctx.regs_mut().pc = val;
            inst.cycles
        }
    );
    ($name:ident, LOAD, $dst:tt, $src:tt) => (
        fn $name<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
            let val = read_arg!(ctx, inst, $src);
            write_arg!(ctx, $dst, val);
            ctx.regs_mut().inc_pc(inst.size);
            inst.cycles
        }
    );
}

exec_func!(exec_add_hl_bc, ADD16, HL, BC);
exec_func!(exec_dec_b, DEC8, B);
exec_func!(exec_dec_c, DEC8, C);
exec_func!(exec_dec_bc, DEC16, BC);
exec_func!(exec_inc_b, INC8, B);
exec_func!(exec_inc_c, INC8, C);
exec_func!(exec_inc_bc, INC16, BC);
exec_func!(exec_jp_l16, JP, L16);
exec_func!(exec_ld_a_indbc, LOAD, A, INDBC);
exec_func!(exec_ld_b_l8, LOAD, B, L8);
exec_func!(exec_ld_c_l8, LOAD, C, L8);
exec_func!(exec_ld_bc_l16, LOAD, BC, L16);
exec_func!(exec_ld_indbc_a, LOAD, INDBC, A);

fn exec_nop<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
    ctx.regs_mut().inc_pc(inst.size);
    inst.cycles
}

fn exec_exaf<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
    ctx.regs_mut().swap_af();
    ctx.regs_mut().inc_pc(inst.size);
    inst.cycles
}

fn exec_rlca<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
    let orig = read_arg!(ctx, inst, A);
    let carry = orig >> 7;
    let dest = (orig << 1) | carry;
    write_arg!(ctx, A, dest);
    ctx.regs_mut().inc_pc(inst.size);
    let flags = flags_apply!(ctx.regs().flags(), C:[carry > 0] H:0 N:0);
    ctx.regs_mut().set_flags(flags);

    inst.cycles
}

fn exec_rrca<C: Context>(inst: &Inst, ctx: &mut C) -> Cycles {
    let orig = read_arg!(ctx, inst, A);
    let carry = orig << 7;
    let dest = (orig >> 1) | carry;
    write_arg!(ctx, A, dest);
    ctx.regs_mut().inc_pc(inst.size);
    let flags = flags_apply!(ctx.regs().flags(), C:[carry > 0] H:0 N:0);
    ctx.regs_mut().set_flags(flags);
    inst.cycles
}

#[cfg(all(feature = "nightly", test))]
mod bench {
    use test;
    use test::Bencher;

    use cpu::z80;

    use super::*;

    #[bench]
    fn bench_exec_100_nops(b: &mut Bencher) {
        exec_inst(b, &inst!(NOP));
    }

    #[bench]
    fn bench_exec_100_add16(b: &mut Bencher) {
        exec_inst(b, &inst!(ADD HL, BC));
    }

    fn exec_inst(b: &mut Bencher, inst: &Inst) {
        let mem = z80::MemoryBank::new();
        let mut cpu = z80::CPU::new(mem);
        b.iter(|| {
            for _ in 1..100 {
                test::black_box(execute(inst, &mut cpu));
            }
        })
    }
}
