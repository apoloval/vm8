use byteorder::LittleEndian;

use bus::{Bus, ReadFromBytes, WriteFromBytes};
use cpu::z80::{Cycles, MemoryBus, Registers};
use cpu::z80::alu::ALU;

// Context trait defines a context where instructions are executed
pub trait Context {
    type Mem: MemoryBus;
    fn alu(&self) -> &ALU;
    fn regs(&self) -> &Registers;
    fn regs_mut(&mut self) -> &mut Registers;
    fn mem(&self) -> &Self::Mem;
    fn mem_mut(&mut self) -> &mut Self::Mem;

    fn read_from_pc(&self, offset: usize) -> u8 {
        let pc = self.regs().pc();
        let pos = ((pc as usize) + offset) as u16;
        self.mem().read_from(pos)
    }
}

pub fn exec_step<CTX: Context>(ctx: &mut CTX) -> Cycles {
    let pc = ctx.regs().pc();
    let opcode = ctx.read_from_pc(0);
    match opcode {
        0x00 => { ctx.exec_nop();               04 },
        0x01 => { ctx.exec_ld::<BC, L16>();     10 },
        0x02 => { ctx.exec_ld::<IND_BC, A>();   07 },
        0x03 => { ctx.exec_inc16::<BC>();       06 },
        0x04 => { ctx.exec_inc8::<B>();         04 },
        0x05 => { ctx.exec_dec8::<B>();         04 },
        0x06 => { ctx.exec_ld::<B, L8>();       07 },
        0x07 => { ctx.exec_rlca();              04 },
        0x08 => { ctx.exec_exaf();              04 },
        0x09 => { ctx.exec_add16::<HL, BC>();   11 },
        0x0a => { ctx.exec_ld::<A, IND_BC>();   07 },
        0x0b => { ctx.exec_dec16::<BC>();       06 },
        0x0c => { ctx.exec_inc8::<C>();         04 },
        0x0d => { ctx.exec_dec8::<C>();         04 },
        0x0e => { ctx.exec_ld::<C, L8>();       07 },
        0x0f => { ctx.exec_rrca();              04 },
        0x10 => { if ctx.exec_djnz() { 13 } else { 8 } },
        0x11 => { ctx.exec_ld::<DE, L16>();     10 },
        0x12 => { ctx.exec_ld::<IND_DE, A>();   07 },
        0x13 => { ctx.exec_inc16::<DE>();       06 },
        0x14 => { ctx.exec_inc8::<D>();         04 },
        0x15 => { ctx.exec_dec8::<D>();         04 },
        0x16 => { ctx.exec_ld::<D, L8>();       07 },
        0x17 => { ctx.exec_rla();               04 },
        0x18 => { ctx.exec_jr::<L8>();          12 },
        0x19 => { ctx.exec_add16::<HL, DE>();   11 },
        0x1a => { ctx.exec_ld::<A, IND_DE>();   07 },
        0x1b => { ctx.exec_dec16::<DE>();       06 },
        0x1c => { ctx.exec_inc8::<E>();         04 },
        0x1d => { ctx.exec_dec8::<E>();         04 },
        0x1e => { ctx.exec_ld::<E, L8>();       07 },
        0x1f => { ctx.exec_rra();               04 },
        0x20 => { ctx.exec_jr_cond::<NZFLAG, L8>() },
        0x21 => { ctx.exec_ld::<HL, L16>();     10 },
        0x22 => { ctx.exec_ld::<IND16_L16, HL>(); 16 },
        0x23 => { ctx.exec_inc16::<HL>();       06 },
        0x24 => { ctx.exec_inc8::<H>();         04 },
        0x25 => { ctx.exec_dec8::<H>();         04 },
        0x26 => { ctx.exec_ld::<H, L8>();       07 },
        0x27 => { ctx.exec_daa();               04 },
        0x28 => { ctx.exec_jr_cond::<ZFLAG, L8>() },
        0x29 => { ctx.exec_add16::<HL, HL>();   11 },
        0x2a => { ctx.exec_ld::<HL, IND16_L16>(); 16 },
        0x2b => { ctx.exec_dec16::<HL>();       06 },
        0x2c => { ctx.exec_inc8::<L>();         04 },
        0x2d => { ctx.exec_dec8::<L>();         04 },
        0x2e => { ctx.exec_ld::<L, L8>();       07 },
        0x2f => { ctx.exec_cpl();               04 },
        0x30 => { ctx.exec_jr_cond::<NCFLAG, L8>() },
        0x31 => { ctx.exec_ld::<SP, L16>();     10 },
        0x32 => { ctx.exec_ld::<IND8_L16, A>(); 13 },

        0xc3 => { ctx.exec_jp::<L16>();         10 },
        _ => unimplemented!("cannot execute illegal instruction with opcode 0x{:x}", opcode),
    }
}

/********************************************************/

trait Execute : Context + Sized {
    fn exec_add16<D: Src16 + Dest16, S: Src16>(&mut self) {
        let (a, a_size) = D::read_arg(self);
        let (b, b_size) = S::read_arg(self);

        let c = (a as u32) + (b as u32);
        D::write_arg(self, c as u16);
        self.regs_mut().inc_pc(1 + a_size + b_size);

        let flags = flags_apply!(self.regs().flags(),
            C:[c>0xffff]
            H:[((a & 0x0fff) + (b & 0x0fff)) & 0x1000 != 0]
            N:0);
        self.regs_mut().set_flags(flags);
    }

    fn exec_cpl(&mut self) {
        let a = self.regs().a();
        self.regs_mut().set_a(!a);

        let mut flags = self.regs().flags();
        flags = flags_apply!(flags, H:1 N:1);
        self.regs_mut().set_flags(flags);
    }

    fn exec_daa(&mut self) {
        let prev_a = self.regs().a();
        let mut a = prev_a;
        let mut flags = self.regs().flags();
        if flag!(N, flags) == 0 {
            if flag!(H, flags) == 1 || a & 0x0f > 0x09 {
                a = self.alu().add8(a, 0x06);
            }
            if flag!(C, flags) == 1 || a > 0x99 {
                a = self.alu().add8(a, 0x60);
            }
        } else {
            if flag!(H, flags) == 1 || a & 0x0f > 0x09 {
                let (r, _) = self.alu().sub8(a, 0x06);
                a = r;
            }
            if flag!(C, flags) == 1 || a > 0x99 {
                let (r, _) = self.alu().sub8(a, 0x60);
                a = r;
            }
        }
        self.regs_mut().set_a(a);
        self.regs_mut().inc_pc(1);

        flags = flags_apply!(flags,
            S:[a & 0x80 > 0]
            Z:[a == 0]
            H:[(a ^ prev_a) & 0x10 > 0]
            C:[flag!(C, flags) > 0 || prev_a > 0x99]
        );
        self.regs_mut().set_flags(flags);
    }

    fn exec_dec8<D: Src8 + Dest8>(&mut self) {
        let (dest, nbytes) = D::read_arg(self);
        let mut flags = self.regs().flags();
        let result = self.alu().dec8_with_flags(dest, &mut flags);
        D::write_arg(self, result);
        self.regs_mut().inc_pc(1 + nbytes);
        self.regs_mut().set_flags(flags);
    }

    fn exec_dec16<D: Src16 + Dest16>(&mut self) {
        let (dest, nbytes) = D::read_arg(self);
        let result = self.alu().sub16(dest, 1);
        D::write_arg(self, result);
        self.regs_mut().inc_pc(1 + nbytes);
    }

    fn exec_djnz(&mut self) -> bool {
        let (b, _) = self.alu().sub8(self.regs().b(), 1);
        self.regs_mut().set_b(b);
        if b > 0 {
            let s = self.read_from_pc(1);
            let pc = self.regs().pc();
            self.regs_mut().inc_pc8(s);
            true
        } else {
            self.regs_mut().inc_pc(2);
            false
        }
    }

    fn exec_exaf(&mut self) {
        self.regs_mut().swap_af();
        self.regs_mut().inc_pc(1);
    }

    fn exec_inc8<D: Src8 + Dest8>(&mut self) {
        let (dest, nbytes) = D::read_arg(self);
        let mut flags = self.regs().flags();
        let result = self.alu().inc8_with_flags(dest, &mut flags);
        D::write_arg(self, result);
        self.regs_mut().inc_pc(1 + nbytes);
        self.regs_mut().set_flags(flags);
    }

    fn exec_inc16<D: Src16 + Dest16>(&mut self) {
        let (dest, nbytes) = D::read_arg(self);
        let result = (dest as u32 + 1) as u16;
        D::write_arg(self, result);
        self.regs_mut().inc_pc(1 + nbytes);
    }

    fn exec_jp<S: Src16>(&mut self) {
        let (dest, _) = S::read_arg(self);
        self.regs_mut().set_pc(dest);
    }

    fn exec_jr<S: Src8>(&mut self) {
        let (dest, _) = S::read_arg(self);
        self.regs_mut().inc_pc8(dest);
    }

    fn exec_jr_cond<C: Cond, S: Src8>(&mut self) -> usize {
        let cond = C::condition_met(self);
        if cond {
            let (dest, _) = S::read_arg(self);
            self.regs_mut().inc_pc8(dest);
            12
        } else {
            self.regs_mut().inc_pc8(2);
            7
        }
    }

    fn exec_ld<D: Dest, S: Src>(&mut self)
    where D: Dest<Item=S::Item> {
        let (src, src_nbytes) = S::read_arg(self);
        let dst_nbytes = D::write_arg(self, src);
        self.regs_mut().inc_pc(1 + src_nbytes + dst_nbytes);
    }

    fn exec_nop(&mut self) {
        self.regs_mut().inc_pc(1);
    }

    fn exec_rla(&mut self) {
        let mut flags = self.regs().flags();
        let orig = self.regs().a();
        let carry = self.regs().flag_c();
        let dest = self.alu().rotate_left(orig, carry, &mut flags);
        self.regs_mut().set_a(dest);
        self.regs_mut().inc_pc(1);
        self.regs_mut().set_flags(flags);
    }

    fn exec_rlca(&mut self) {
        let mut flags = self.regs().flags();
        let orig = self.regs().a();
        let carry = (orig & 0x80) >> 7;
        let dest = self.alu().rotate_left(orig, carry, &mut flags);
        self.regs_mut().set_a(dest);
        self.regs_mut().inc_pc(1);
        self.regs_mut().set_flags(flags);
    }

    fn exec_rra(&mut self) {
        let mut flags = self.regs().flags();
        let orig = self.regs().a();
        let carry = self.regs().flag_c();
        let dest = self.alu().rotate_right(orig, carry, &mut flags);
        self.regs_mut().set_a(dest);
        self.regs_mut().inc_pc(1);
        self.regs_mut().set_flags(flags);
    }

    fn exec_rrca(&mut self) {
        let mut flags = self.regs().flags();
        let orig = self.regs().a();
        let carry = orig & 0x01;
        let dest = self.alu().rotate_right(orig, carry, &mut flags);
        self.regs_mut().set_a(dest);
        self.regs_mut().inc_pc(1);
        self.regs_mut().set_flags(flags);
    }

    fn alu_add8(a: u8, b: u8, flags: u8) -> (u8, u8) {
        let c = ((a as u16) + (b as u16)) as u8;
        let new_flags = flags_apply!(flags,
            S:[(c & 0x80) != 0]
            Z:[c == 0]
            H:[((a & 0x0f) + (b & 0x0f)) & 0x10 != 0]
            PV:[(a ^ b ^ 0x80) & (b ^ c) & 0x80 != 0]
            N:0
            C:[c < a]);
        (c as u8, new_flags)
    }
}

impl<T> Execute for T where T: Context + Sized {}


/********************************************************/


type FetchedBytes = usize;
type Operand<T> = (T, FetchedBytes);

trait Src {
    type Item;
    fn read_arg<C: Context>(ctx: &C) -> Operand<Self::Item>;
}

trait Src8 : Src<Item=u8> {}
impl<T> Src8 for T where T: Src<Item=u8> {}

trait Src16 : Src<Item=u16> {}
impl<T> Src16 for T where T: Src<Item=u16> {}

trait Dest {
    type Item;
    fn write_arg<C: Context>(ctx: &mut C, val: Self::Item) -> FetchedBytes;
}

trait Dest8 : Dest<Item=u8> {}
impl<T> Dest8 for T where T: Dest<Item=u8> {}

trait Dest16 : Dest<Item=u16> {}
impl<T> Dest16 for T where T: Dest<Item=u16> {}

macro_rules! def_reg8_arg {
    ($reg:tt, $r8r:ident, $r8w:ident) => (
        struct $reg;

        impl Src for $reg {
            type Item = u8;

            #[inline]
            fn read_arg<C: Context>(ctx: &C) -> Operand<u8> {
                (ctx.regs().$r8r(), 0)
            }
        }

        impl Dest for $reg {
            type Item = u8;

            #[inline]
            fn write_arg<C: Context>(ctx: &mut C, val: u8) -> FetchedBytes {
                ctx.regs_mut().$r8w(val);
                0
            }
        }
    );
}

macro_rules! def_reg16_arg {
    ($reg:tt, $r16r:ident, $r16w:ident) => (
        struct $reg;

        impl Src for $reg {
            type Item = u16;

            #[inline]
            fn read_arg<C: Context>(ctx: &C) -> Operand<u16> {
                (ctx.regs().$r16r(), 0)
            }
        }

        impl Dest for $reg {
            type Item = u16;

            #[inline]
            fn write_arg<C: Context>(ctx: &mut C, val: u16) -> FetchedBytes {
                ctx.regs_mut().$r16w(val);
                0
            }
        }
    );
}

macro_rules! def_indreg16_arg {
    ($reg:tt, $r16:ident) => (
        struct $reg;

        impl Src for $reg {
            type Item = u8;

            #[inline]
            fn read_arg<C: Context>(ctx: &C) -> Operand<u8> {
                let addr = ctx.regs().$r16();
                let data = ctx.mem().read_from(addr);
                (data, 0)
            }
        }

        impl Dest for $reg {
            type Item = u8;

            #[inline]
            fn write_arg<C: Context>(ctx: &mut C, val: u8) -> FetchedBytes {
                let addr = ctx.regs().$r16();
                ctx.mem_mut().write_to(addr, val);
                0
            }
        }
    );
}

def_reg8_arg!(A, a, set_a);
def_reg8_arg!(B, b, set_b);
def_reg8_arg!(C, c, set_c);
def_reg8_arg!(D, d, set_d);
def_reg8_arg!(E, e, set_e);
def_reg8_arg!(H, h, set_h);
def_reg8_arg!(L, l, set_l);

def_reg16_arg!(AF, af, set_af);
def_reg16_arg!(BC, bc, set_bc);
def_reg16_arg!(DE, de, set_de);
def_reg16_arg!(HL, hl, set_hl);
def_reg16_arg!(SP, sp, set_sp);

def_indreg16_arg!(IND_BC, bc);
def_indreg16_arg!(IND_DE, de);
def_indreg16_arg!(IND_HL, hl);

struct IND8_L16;

impl Src for IND8_L16 {
    type Item = u8;

    #[inline]
    fn read_arg<C: Context>(ctx: &C) -> Operand<u8> {
        let pc = ctx.regs().pc();
        let (addr, _) = ctx.alu().add16(pc, 1);
        let ind = ctx.mem().read_word_from::<LittleEndian>(addr);
        let data = ctx.mem().read_from(ind);
        (data, 2)
    }
}

impl Dest for IND8_L16 {
    type Item = u8;

    #[inline]
    fn write_arg<C: Context>(ctx: &mut C, val: u8) -> FetchedBytes {
        let pc = ctx.regs().pc();
        let (addr, _) = ctx.alu().add16(pc, 1);
        let ind = ctx.mem().read_word_from::<LittleEndian>(addr);
        ctx.mem_mut().write_to(ind, val);
        2
    }
}

struct IND16_L16;

impl Src for IND16_L16 {
    type Item = u16;

    #[inline]
    fn read_arg<C: Context>(ctx: &C) -> Operand<u16> {
        let pc = ctx.regs().pc();
        let (addr, _) = ctx.alu().add16(pc, 1);
        let ind = ctx.mem().read_word_from::<LittleEndian>(addr);
        let data = ctx.mem().read_word_from::<LittleEndian>(ind);
        (data, 2)
    }
}

impl Dest for IND16_L16 {
    type Item = u16;

    #[inline]
    fn write_arg<C: Context>(ctx: &mut C, val: u16) -> FetchedBytes {
        let pc = ctx.regs().pc();
        let (addr, _) = ctx.alu().add16(pc, 1);
        let ind = ctx.mem().read_word_from::<LittleEndian>(addr);
        ctx.mem_mut().write_word_to::<LittleEndian>(ind, val);
        2
    }
}

struct L8;
impl Src for L8 {
    type Item = u8;

    #[inline]
    fn read_arg<C: Context>(ctx: &C) -> Operand<u8> {
        let pc = ctx.regs().pc();
        (ctx.mem().read_from(pc + 1), 1)
    }
}

struct L16;
impl Src for L16 {
    type Item = u16;

    #[inline]
    fn read_arg<C: Context>(ctx: &C) -> Operand<u16> {
        let pc = ctx.regs().pc();
        (ctx.mem().read_word_from::<LittleEndian>(pc + 1), 2)
    }
}

trait Cond {
    fn condition_met<C: Context>(ctx: &C) -> bool;
}

macro_rules! def_cond {
    ($name:ident, $flagget:ident, $flagvalue:expr) => {
        struct $name;

        impl Cond for $name {
            fn condition_met<C: Context>(ctx: &C) -> bool {
                let flag = ctx.regs().$flagget();
                flag == $flagvalue
            }
        }
    }
}

def_cond!(ZFLAG, flag_z, 1);
def_cond!(NZFLAG, flag_z, 0);
def_cond!(CFLAG, flag_c, 1);
def_cond!(NCFLAG, flag_c, 0);

/********************************************************/

#[cfg(test)]
mod test {
    use std::fmt;
    use std::io::Write;

    use rand;
    use rand::prelude::*;

    use cpu::z80;

    use super::*;

    /********************/
    /* 8-Bit Load Group */
    /********************/

    macro_rules! test_ld_indreg_a {
        ($fname:ident, $regname:ident, $regset:ident) => {
            #[test]
            fn $fname() {
                let mut test = ExecTest::for_inst(&inst!(LD ($regname), A));
                test.assert_behaves_like_ld(0,
                    |val, cpu| {
                        cpu.regs_mut().set_a(val);
                        cpu.regs_mut().$regset(0x1234);
                    },
                    |cpu| cpu.mem().read_from(0x1234),
                );
            }
        }
    }

    test_ld_indreg_a!(test_exec_ld_indbc_a, BC, set_bc);
    test_ld_indreg_a!(test_exec_ld_indde_a, DE, set_de);

    macro_rules! test_ld_a_indl16 {
        ($fname:ident, $regname:ident, $regset:ident) => {
            #[test]
            fn $fname() {
                let mut test = ExecTest::for_inst(&inst!(LD A, ($regname)));
                test.assert_behaves_like_ld(0,
                    |val, cpu| {
                        cpu.mem_mut().write_to(0x1234, val);
                        cpu.regs_mut().$regset(0x1234);
                    },
                    |cpu| cpu.regs().a(),
                );
            }
        };
    }

    test_ld_a_indl16!(test_exec_ld_a_indbc, BC, set_bc);
    test_ld_a_indl16!(test_exec_ld_a_indde, DE, set_de);

    macro_rules! test_ld_indl16_r8 {
        ($fname:ident, $regname:ident, $regset:ident) => {
            #[test]
            fn $fname() {
                let mut test = ExecTest::for_inst(&inst!(LD (0x1234), $regname));
                test.assert_behaves_like_ld(2,
                    |val, cpu| cpu.regs_mut().$regset(val),
                    |cpu| cpu.mem().read_from(0x1234),
                );
            }
        }
    }

    test_ld_indl16_r8!(test_exec_ld_indl16_a, A, set_a);

    macro_rules! test_ld_r8_l8 {
        ($fname:ident, $regname:ident, $regget:ident) => {
            #[test]
            fn $fname() {
                let mut test = ExecTest::new();
                test.assert_behaves_like_ld(1,
                    |val, cpu| { Write::write(cpu.mem_mut(), &inst!(LD $regname, val)).unwrap(); },
                    |cpu| cpu.regs().$regget(),
                );
            }

        }
    }

    test_ld_r8_l8!(test_exec_ld_b_l8, B, b);
    test_ld_r8_l8!(test_exec_ld_c_l8, C, c);
    test_ld_r8_l8!(test_exec_ld_d_l8, D, d);
    test_ld_r8_l8!(test_exec_ld_e_l8, E, e);
    test_ld_r8_l8!(test_exec_ld_h_l8, H, h);
    test_ld_r8_l8!(test_exec_ld_l_l8, L, l);

    /*********************/
    /* 16-Bit Load Group */
    /*********************/

    macro_rules! test_ld_r16_l16 {
        ($fname:ident, $regname:ident, $regget:ident) => {
            #[test]
            fn $fname() {
                let mut test = ExecTest::new();
                test.assert_behaves_like_ld(2,
                    |val, cpu| { Write::write(cpu.mem_mut(), &inst!(LD $regname, val)).unwrap(); },
                    |cpu| cpu.regs().$regget(),
                );
            }
        }
    }

    test_ld_r16_l16!(test_exec_ld_bc_l16, BC, bc);
    test_ld_r16_l16!(test_exec_ld_de_l16, DE, de);
    test_ld_r16_l16!(test_exec_ld_hl_l16, HL, hl);
    test_ld_r16_l16!(test_exec_ld_sp_l16, SP, sp);

    macro_rules! test_ld_indl16_r16 {
        ($fname:ident, $regname:ident, $regset:ident) => {
            #[test]
            fn $fname() {
                let mut test = ExecTest::for_inst(&inst!(LD (0x1234), $regname));
                test.assert_behaves_like_ld(2,
                    |val, cpu| cpu.regs_mut().$regset(val),
                    |cpu| cpu.mem().read_word_from::<LittleEndian>(0x1234),
                );
            }
        }
    }

    test_ld_indl16_r16!(test_exec_ld_indl16_hl, HL, set_hl);

    macro_rules! test_ld_r16_indl16 {
        ($fname:ident, $regname:ident, $regget:ident) => {
            #[test]
            fn $fname() {
                let mut test = ExecTest::for_inst(&inst!(LD $regname, (0x1234)));
                test.assert_behaves_like_ld(2,
                    |val, cpu| cpu.mem_mut().write_word_to::<LittleEndian>(0x1234, val),
                    |cpu| cpu.regs().$regget(),
                );
            }
        }
    }

    test_ld_r16_indl16!(test_exec_ld_hl_indl16, HL, hl);

    /**********************************************/
    /* Exchange, Block Transfer, and Search Group */
    /**********************************************/

    #[test]
    fn test_exec_exaf() {
        let mut test = ExecTest::for_inst(&inst!(EX AF, AF_));
        let input = 0x12;
        let input_ = 0x34;

        test.cpu.regs_mut().set_af(input);
        test.cpu.regs_mut().set_af_(input_);
        test.exec_step();

        let expected = input_;
        let expected_ = input;
        let given = test.cpu.regs().af();
        let given_ = test.cpu.regs().af_();
        assert_result!(HEX16, "program counter", 0x0001, test.cpu.regs().pc());
        assert_result!(HEX8, "AF", expected, given);
        assert_result!(HEX8, "AF'", expected_, given_);
    }

    /**************************/
    /* 8-Bit Arithmetic group */
    /**************************/

    macro_rules! test_inc_reg8 {
        ($fname:ident, $regname:ident, $regget:ident, $regset:ident) => {
            #[test]
            fn $fname() {
                let mut test = ExecTest::for_inst(&inst!(INC $regname));
                test.assert_behaves_like_inc8(
                    |v, cpu| cpu.regs_mut().$regset(v),
                    |cpu| cpu.regs().$regget(),
                );
            }
        }
    }

    test_inc_reg8!(test_exec_inc_c, C, c, set_c);
    test_inc_reg8!(test_exec_inc_d, D, d, set_d);
    test_inc_reg8!(test_exec_inc_e, E, e, set_e);
    test_inc_reg8!(test_exec_inc_h, H, h, set_h);
    test_inc_reg8!(test_exec_inc_l, L, l, set_l);

    macro_rules! test_dec_reg8 {
        ($fname:ident, $regname:ident, $regget:ident, $regset:ident) => {
            #[test]
            fn $fname() {
                let mut test = ExecTest::for_inst(&inst!(DEC $regname));
                test.assert_behaves_like_dec8(
                    |v, cpu| cpu.regs_mut().$regset(v),
                    |cpu| cpu.regs().$regget(),
                );
            }
        }
    }

    test_dec_reg8!(test_exec_dec_b, B, b, set_b);
    test_dec_reg8!(test_exec_dec_c, C, c, set_c);
    test_dec_reg8!(test_exec_dec_d, D, d, set_d);
    test_dec_reg8!(test_exec_dec_e, E, e, set_e);
    test_dec_reg8!(test_exec_dec_h, H, h, set_h);
    test_dec_reg8!(test_exec_dec_l, L, l, set_l);

    /*****************************************************/
    /* General-Purpose Arithmetic and CPU Control Groups */
    /*****************************************************/

    #[test]
    fn test_exec_cpl() {
        let mut test = ExecTest::for_inst(&inst!(CPL));
        test.cpu.regs_mut().set_a(0x42);
        test.exec_step();
        
        let actual = test.cpu.regs().a();
        let expected = 0xbd;
        assert_result!(HEX8, "A", expected, actual);

        test.assert_hflag_if("CPL", true);
        test.assert_nflag_if("CPL", true);
    }

    #[test]
    fn test_exec_daa() {
        struct Case {
            name: &'static str,
            pre_a: u8,
            pre_flags: u8,
            expected_a: u8,
            expected_flags: u8,
        }
        let mut test = ExecTest::for_inst(&inst!(DAA));
        table_test!(
            &[
            Case {
                name: "Already adjusted",
                pre_a: 0x42,
                pre_flags: flags_apply!(0, N:0 H:0 C:0),
                expected_a: 0x42,
                expected_flags: 0,
            },
            Case {
                name: "Need to adjust low nibble after add",
                pre_a: 0x4d,
                pre_flags: flags_apply!(0, N:0 H:0 C:0),
                expected_a: 0x53,
                expected_flags: flags_apply!(0, N:0 H:1 C:0),
            },
            Case {
                name: "Need to adjust low nibble after subtract",
                pre_a: 0x4d,
                pre_flags: flags_apply!(0, N:1 H:0 C:0),
                expected_a: 0x47,
                expected_flags: flags_apply!(0, N:1 H:0 C:0),
            },
            Case {
                name: "Need to adjust high nibble after add",
                pre_a: 0xd4,
                pre_flags: flags_apply!(0, N:0 H:0 C:0),
                expected_a: 0x34,
                expected_flags: flags_apply!(0, N:0 H:0 C:1),
            },
            Case {
                name: "Need to adjust high nibble after subtract",
                pre_a: 0xd4,
                pre_flags: flags_apply!(0, N:1 H:0 C:0),
                expected_a: 0x74,
                expected_flags: flags_apply!(0, N:1 H:0 C:1),
            },
            ],
            |case: &Case| {
                test.cpu.regs_mut().set_a(case.pre_a);
                test.cpu.regs_mut().set_flags(case.pre_flags);
                test.exec_step();

                let given_a = test.cpu.regs().a();
                let given_flags = test.cpu.regs().flags();
                assert_result!(HEX16, "program counter", 0x0001, test.cpu.regs().pc());
                assert_result!(HEX8, "register A", case.expected_a, given_a);
                assert_result!(BIN8, "flags", case.expected_flags, given_flags);
            }
        );
    }

    #[test]
    fn test_exec_nop() {
        let mut test = ExecTest::for_inst(&inst!(NOP));
        test.exec_step();
        assert_eq!(0x0001, test.cpu.regs().pc());
        test.assert_all_flags_unaffected("nop");
    }

    /***************************/
    /* 16-Bit Arithmetic group */
    /***************************/

    macro_rules! test_inc_reg16 {
        ($fname:ident, $regname:ident, $regget:ident, $regset:ident) => {
            #[test]
            fn $fname() {
                let mut test = ExecTest::for_inst(&inst!(INC $regname));
                test.assert_behaves_like_inc16(
                    |v, cpu| cpu.regs_mut().$regset(v),
                    |cpu| cpu.regs().$regget(),
                );
            }
        }
    }

    test_inc_reg16!(test_exec_inc_bc, BC, bc, set_bc);
    test_inc_reg16!(test_exec_inc_de, DE, de, set_de);
    test_inc_reg16!(test_exec_inc_hl, HL, hl, set_hl);

    macro_rules! test_dec_reg16 {
        ($fname:ident, $regname:ident, $regget:ident, $regset:ident) => {
            #[test]
            fn $fname() {
                let mut test = ExecTest::for_inst(&inst!(DEC $regname));
                test.assert_behaves_like_dec16(
                    |v, cpu| cpu.regs_mut().$regset(v),
                    |cpu| cpu.regs().$regget(),
                );
            }
        }
    }

    test_dec_reg16!(test_exec_dec_bc, BC, bc, set_bc);
    test_dec_reg16!(test_exec_dec_de, DE, de, set_de);
    test_dec_reg16!(test_exec_dec_hl, HL, hl, set_hl);

    macro_rules! test_add_reg16_reg16 {
        ($fname:ident, $dstname:ident, $srcname:ident,
         $dstget:ident, $dstset:ident, $srcset:ident) => {
            #[test]
            fn $fname() {
                let mut test = ExecTest::for_inst(&inst!(ADD $dstname, $srcname));
                test.asset_behaves_like_add16(
                    |a, b, cpu| {
                        cpu.regs_mut().$dstset(a);
                        cpu.regs_mut().$srcset(b);
                    },
                    |cpu| cpu.regs().$dstget(),
                );
            }
        }
    }

    test_add_reg16_reg16!(test_exec_add_hl_bc, HL, BC, hl, set_hl, set_bc);
    test_add_reg16_reg16!(test_exec_add_hl_de, HL, DE, hl, set_hl, set_de);

    #[test]
    fn test_exec_add_hl_hl() {
        struct Case {
            name: &'static str,
            a: u16,
            expected: u16,
            expected_flags: fn(u8) -> u8,
        }
        let mut test = ExecTest::for_inst(&inst!(ADD HL, HL));
        table_test!(&[
            Case {
                name: "Regular case",
                a: 0x1245,
                expected: 0x248a,
                expected_flags: |f| flags_apply!(f, H:0 N:0 C:0),
            },
            Case {
                name: "Half carry",
                a: 0x1f45,
                expected: 0x3e8a,
                expected_flags: |f| flags_apply!(f, H:1 N:0 C:0),
            },
            Case {
                name: "Carry",
                a: 0xff45,
                expected: 0xfe8a,
                expected_flags: |f| flags_apply!(f, H:1 N:0 C:1),
            },
        ], |case: &Case| {
            test.cpu.regs_mut().set_hl(case.a);
            test.exec_step();
            assert_result!(HEX16, "program counter", 0x0001, test.cpu.regs().pc());
            let actual = test.cpu.regs().hl();
            let expected_flags = (case.expected_flags)(test.cpu.regs().flags());
            let actual_flags = test.cpu.regs().flags();
            assert_result!(HEX16, "dest", case.expected, actual);
            assert_result!(BIN8, "flags", expected_flags, actual_flags);
        });
    }

    /**************************/
    /* Rotate and Shift Group */
    /**************************/

    #[test]
    fn test_exec_rlca() {
        struct Case {
            name: &'static str,
            a: u8,
            expected: u8,
            expected_flags: fn(u8) -> u8,
        }
        let mut test = ExecTest::for_inst(&inst!(RLCA));
        table_test!(&[
            Case {
                name: "No carry",
                a: 0x12,
                expected: 0x24,
                expected_flags: |f| flags_apply!(f, H:0 N:0 C: 0),
            },
            Case {
                name: "Carry",
                a: 0xc8,
                expected: 0x91,
                expected_flags: |f| flags_apply!(f, H:0 N:0 C: 1),
            },
        ], |case: &Case| {
            let prev_flags = test.cpu.regs().flags();
            test.cpu.regs_mut().set_a(case.a);
            test.exec_step();

            let actual = test.cpu.regs().a();
            let expected_flags = (case.expected_flags)(prev_flags);
            let actual_flags = test.cpu.regs().flags();
            assert_result!(HEX16, "program counter", 0x0001, test.cpu.regs().pc());
            assert_result!(HEX8, "dest", case.expected, actual);
            assert_result!(BIN8, "flags", expected_flags, actual_flags);
        });
    }

    #[test]
    fn test_exec_rrca() {
        struct Case {
            name: &'static str,
            a: u8,
            expected: u8,
            expected_flags: fn(u8) -> u8,
        }
        let mut test = ExecTest::for_inst(&inst!(RRCA));
        table_test!(&[
            Case {
                name: "No carry",
                a: 0x24,
                expected: 0x12,
                expected_flags: |f| flags_apply!(f, H:0 N:0 C: 0),
            },
            Case {
                name: "Carry",
                a: 0x91,
                expected: 0xc8,
                expected_flags: |f| flags_apply!(f, H:0 N:0 C: 1),
            },
        ], |case: &Case| {
            let prev_flags = test.cpu.regs().flags();
            test.cpu.regs_mut().set_a(case.a);
            test.exec_step();

            let actual = test.cpu.regs().a();
            let expected_flags = (case.expected_flags)(prev_flags);
            let actual_flags = test.cpu.regs().flags();
            assert_result!(HEX16, "program counter", 0x0001, test.cpu.regs().pc());
            assert_result!(HEX8, "dest", case.expected, actual);
            assert_result!(BIN8, "flags", expected_flags, actual_flags);
        });
    }

    #[test]
    fn test_exec_rla() {
        struct Case {
            name: &'static str,
            a: u8,
            carry: u8,
            expected: u8,
            expected_flags: fn(u8) -> u8,
        }
        let mut test = ExecTest::for_inst(&inst!(RLA));
        table_test!(&[
            Case {
                name: "No carry",
                a: 0x12,
                carry: 0,
                expected: 0x24,
                expected_flags: |f| flags_apply!(f, H:0 N:0 C: 0),
            },
            Case {
                name: "Carry in",
                a: 0x12,
                carry: 1,
                expected: 0x25,
                expected_flags: |f| flags_apply!(f, H:0 N:0 C: 0),
            },
            Case {
                name: "Carry out",
                a: 0xc8,
                carry: 0,
                expected: 0x90,
                expected_flags: |f| flags_apply!(f, H:0 N:0 C: 1),
            },
            Case {
                name: "Carry inout",
                a: 0xc8,
                carry: 1,
                expected: 0x91,
                expected_flags: |f| flags_apply!(f, H:0 N:0 C: 1),
            },
        ], |case: &Case| {
            let mut prev_flags = test.cpu.regs().flags();
            prev_flags = flags_apply!(prev_flags, C:[case.carry == 1]);
            test.cpu.regs_mut().set_flags(prev_flags);
            test.cpu.regs_mut().set_a(case.a);
            test.exec_step();

            let actual = test.cpu.regs().a();
            let expected_flags = (case.expected_flags)(prev_flags);
            let actual_flags = test.cpu.regs().flags();
            assert_result!(HEX16, "program counter", 0x0001, test.cpu.regs().pc());
            assert_result!(HEX8, "dest", case.expected, actual);
            assert_result!(BIN8, "flags", expected_flags, actual_flags);
        });
    }

    #[test]
    fn test_exec_rra() {
        struct Case {
            name: &'static str,
            a: u8,
            carry: u8,
            expected: u8,
            expected_flags: fn(u8) -> u8,
        }
        let mut test = ExecTest::for_inst(&inst!(RRA));
        table_test!(&[
            Case {
                name: "No carry",
                a: 0x24,
                carry: 0,
                expected: 0x12,
                expected_flags: |f| flags_apply!(f, H:0 N:0 C: 0),
            },
            Case {
                name: "Carry in",
                a: 0x24,
                carry: 1,
                expected: 0x92,
                expected_flags: |f| flags_apply!(f, H:0 N:0 C: 0),
            },
            Case {
                name: "Carry out",
                a: 0x91,
                carry: 0,
                expected: 0x48,
                expected_flags: |f| flags_apply!(f, H:0 N:0 C: 1),
            },
            Case {
                name: "Carry inout",
                a: 0x91,
                carry: 1,
                expected: 0xc8,
                expected_flags: |f| flags_apply!(f, H:0 N:0 C: 1),
            },
        ], |case: &Case| {
            let mut prev_flags = test.cpu.regs().flags();
            prev_flags = flags_apply!(prev_flags, C:[case.carry == 1]);
            test.cpu.regs_mut().set_flags(prev_flags);
            test.cpu.regs_mut().set_a(case.a);
            test.exec_step();

            let actual = test.cpu.regs().a();
            let expected_flags = (case.expected_flags)(prev_flags);
            let actual_flags = test.cpu.regs().flags();
            assert_result!(HEX16, "program counter", 0x0001, test.cpu.regs().pc());
            assert_result!(HEX8, "dest", case.expected, actual);
            assert_result!(BIN8, "flags", expected_flags, actual_flags);
        });
    }

    /**************/
    /* Jump Group */
    /**************/

    #[test]
    fn test_exec_djnz_l8() {
        struct Case {
            name: &'static str,
            input: u8,
            dest: i8,
            expected: u8,
            expected_pc: u16,
        }
        let mut test = ExecTest::new();
        table_test!(&[
            Case {
                name: "Branch forwards",
                input: 10,
                dest: 0x55,
                expected: 09,
                expected_pc: 0x0055,
            },
            Case {
                name: "Branch backwards",
                input: 10,
                dest: -0x10,
                expected: 09,
                expected_pc: 0xfff0,
            },
            Case {
                name: "No branch",
                input: 1,
                dest: 0x55,
                expected: 0,
                expected_pc: 0x0002,
            },
        ], |case: &Case| {
            test.cpu.mem_mut().write(&inst!(DJNZ case.dest as u8)).unwrap();
            test.cpu.regs_mut().set_b(case.input);
            test.exec_step();

            let actual = test.cpu.regs().b();
            let actual_pc = test.cpu.regs().pc();
            assert_result!(HEX8, "B", case.expected, actual);
            assert_result!(HEX16, "program counter", case.expected_pc, actual_pc);

            test.assert_all_flags_unaffected("DJNZ");
        });
    }

    #[test]
    fn test_exec_jr_l8() {
        struct Case {
            name: &'static str,
            dest: i8,
            expected_pc: u16,
        }
        let mut test = ExecTest::new();
        table_test!(&[
            Case {
                name: "Branch forwards",
                dest: 0x55,
                expected_pc: 0x0055,
            },
            Case {
                name: "Branch backwards",
                dest: -0x10,
                expected_pc: 0xfff0,
            },
        ], |case: &Case| {
            test.cpu.mem_mut().write(&inst!(JR case.dest as u8)).unwrap();
            test.exec_step();

            let actual_pc = test.cpu.regs().pc();
            assert_result!(HEX16, "program counter", case.expected_pc, actual_pc);

            test.assert_all_flags_unaffected("JR");
        });
    }

    macro_rules! test_jr_cond_l8 {
        ($fname:ident, $condname:ident, $flagget:ident, $met:expr, $unmet:expr) => {
            #[test]
            fn $fname() {
                struct Case {
                    name: &'static str,
                    dest: i8,
                    branch: bool,
                    expected_pc: u16,
                }
                let mut test = ExecTest::new();
                table_test!(&[
                    Case {
                        name: "Branch forwards",
                        dest: 0x55,
                        branch: true,
                        expected_pc: 0x0055,
                    },
                    Case {
                        name: "Branch backwards",
                        dest: -0x10,
                        branch: true,
                        expected_pc: 0xfff0,
                    },
                    Case {
                        name: "No branch",
                        dest: 0x55,
                        branch: false,
                        expected_pc: 0x0002,
                    },
                ], |case: &Case| {
                    test.cpu.mem_mut().write(&inst!(JR $condname, case.dest as u8)).unwrap();
                    let mut flags = test.cpu.regs().flags();
                    if case.branch { flags = $met(flags); }
                    else { flags = $unmet(flags); }
                    test.cpu.regs_mut().set_flags(flags);
                    test.exec_step();

                    let actual_pc = test.cpu.regs().pc();
                    assert_result!(HEX16, "program counter", case.expected_pc, actual_pc);

                    test.assert_all_flags_unaffected("JR");
                });
            }
        }
    }

    test_jr_cond_l8!(test_exec_jr_nc_l8, NC, flag_c, |f| f & 0b11111110, |f| f | 0b00000001);
    test_jr_cond_l8!(test_exec_jr_nz_l8, NZ, flag_z, |f| f & 0b10111111, |f| f | 0b01000000);
    test_jr_cond_l8!(test_exec_jr_z_l8, Z, flag_z, |f| f | 0b01000000, |f| f & 0b10111111);

    /****************************************/
    /* Test suite for instruction execution */
    /****************************************/

    type CPU = z80::CPU<z80::MemoryBank>;

    struct ExecTest {
        pub cpu: CPU,
        prev_flags: u8,
    }

    trait Data : fmt::Display + fmt::Debug + fmt::UpperHex + Copy + PartialEq {
        fn sample() -> Self;
    }

    impl Data for u8 {
        fn sample() -> u8 { rand::random() }
    }

    impl Data for u16 {
        fn sample() -> u16 { rand::random() }
    }

    impl ExecTest {
        fn new() -> Self {
            let prev_flags = u8::sample() & 0b11010111; // Do not set F5 and F3
            let mem = z80::MemoryBank::new();
            let mut cpu = z80::CPU::new(z80::Options::default(), mem);
            cpu.regs_mut().set_flags(prev_flags);
            Self { cpu, prev_flags }
        }

        fn for_inst(mut inst: &[u8]) -> Self {
            let mut test = Self::new();
            Write::write(test.cpu.mem_mut(), inst).unwrap();
            test
        }

        fn exec_step(&mut self) {
            self.cpu.regs_mut().set_pc(0x0000);
            self.prev_flags = self.cpu.regs().flags();
            exec_step(&mut self.cpu);
        }

        fn assert_behaves_like_ld<S, G, D>(&mut self, opsize: usize, set: S, get: G)
        where S: Fn(D, &mut CPU), G: Fn(&CPU) -> D, D: Data {
            let input = D::sample();
            set(input, &mut self.cpu);

            self.exec_step();

            let output = get(&self.cpu);
            let expected_pc = 1 + opsize as u16;
            let actual_pc = self.cpu.regs().pc();
            let flags = self.cpu.regs().flags();

            assert_result!(HEX16, "program counter", expected_pc, actual_pc);
            assert_result!(HEX16, "dest", input, output);

            self.assert_all_flags_unaffected("LD");
        }

        fn assert_behaves_like_inc8<S, G>(&mut self, set: S, get: G)
        where S: Fn(u8, &mut CPU), G: Fn(&CPU) -> u8 {
            struct Case {
                name: &'static str,
                input: u8,
                expected: u8,
                expected_flags: fn(u8) -> u8,
            }
            table_test!(&[
                Case {
                    name: "Regular case",
                    input: 0x01,
                    expected: 0x02,
                    expected_flags: |f| flags_apply!(f, S:0 Z:0 H:0 PV:0 N:0),
                },
                Case {
                    name: "Half-carry",
                    input: 0x0f,
                    expected: 0x10,
                    expected_flags: |f| flags_apply!(f, S:0 Z:0 H:1 PV:0 N:0),
                },
                Case {
                    name: "Overflow",
                    input: 0x7f,
                    expected: 0x80,
                    expected_flags: |f| flags_apply!(f, S:1 Z:0 H:1 PV:1 N:0),
                },
                Case {
                    name: "Carry",
                    input: 0xff,
                    expected: 0x00,
                    expected_flags: |f| flags_apply!(f, S:0 Z:1 H:1 PV:0 N:0),
                },
            ], |case: &Case| {
                let prev_flags = self.cpu.regs().flags();
                set(case.input, &mut self.cpu);
                self.exec_step();
                let actual = get(&self.cpu);
                let flags = self.cpu.regs().flags();
                let expected_flags = (case.expected_flags)(prev_flags);

                assert_result!(HEX16, "program counter", 0x0001, self.cpu.regs().pc());
                assert_result!(HEX8, "result", case.expected, actual);
                assert_result!(BIN8, "flags", expected_flags, flags);
            });
        }

        fn assert_behaves_like_inc16<S, G>(&mut self, set: S, get: G)
        where S: Fn(u16, &mut CPU), G: Fn(&CPU) -> u16 {
            struct Case {
                name: &'static str,
                input: u16,
                expected: u16,
            }
            table_test!(&[
                Case {
                    name: "Regular case",
                    input: 0x0001,
                    expected: 0x0002,
                },
                Case {
                    name: "Carry",
                    input: 0xffff,
                    expected: 0x0000,
                },
            ], |case: &Case| {
                set(case.input, &mut self.cpu);
                self.exec_step();
                let actual = get(&self.cpu);

                assert_result!(HEX16, "program counter", 0x0001, self.cpu.regs().pc());
                assert_result!(HEX16, "result", case.expected, actual);
                self.assert_all_flags_unaffected("INC (16-bits)");
            });
        }

        fn assert_behaves_like_dec8<S, G>(&mut self, set: S, get: G)
        where S: Fn(u8, &mut CPU), G: Fn(&CPU) -> u8 {
            struct Case {
                name: &'static str,
                input: u8,
                expected: u8,
                expected_flags: fn(u8) -> u8,
            }
            table_test!(&[
                Case {
                    name: "Regular case",
                    input: 0x02,
                    expected: 0x01,
                    expected_flags: |f| flags_apply!(f, S:0 Z:0 H:0 PV:0 N:1),
                },
                Case {
                    name: "Half-carry",
                    input: 0x10,
                    expected: 0x0f,
                    expected_flags: |f| flags_apply!(f, S:0 Z:0 H:1 PV:0 N:1),
                },
                Case {
                    name: "Overflow",
                    input: 0x80,
                    expected: 0x7f,
                    expected_flags: |f| flags_apply!(f, S:0 Z:0 H:1 PV:1 N:1),
                },
                Case {
                    name: "Zero",
                    input: 0x01,
                    expected: 0x00,
                    expected_flags: |f| flags_apply!(f, S:0 Z:1 H:0 PV:0 N:1),
                },
                Case {
                    name: "No carry",
                    input: 0x00,
                    expected: 0xff,
                    expected_flags: |f| flags_apply!(f, S:1 Z:0 H:1 PV:0 N:1),
                },
            ], |case: &Case| {
                let expected_flags = (case.expected_flags)(self.prev_flags);
                set(case.input, &mut self.cpu);
                self.exec_step();
                let actual = get(&self.cpu);
                let flags = self.cpu.regs().flags();

                assert_result!(HEX16, "program counter", 0x0001, self.cpu.regs().pc());
                assert_result!(HEX8, "result", case.expected, actual);
                assert_result!(BIN8, "flags", expected_flags, flags);
            });
        }

        fn assert_behaves_like_dec16<S, G>(&mut self, set: S, get: G)
        where S: Fn(u16, &mut CPU), G: Fn(&CPU) -> u16 {
            struct Case {
                name: &'static str,
                input: u16,
                expected: u16,
            }
            table_test!(&[
                Case {
                    name: "Regular case",
                    input: 0x0002,
                    expected: 0x0001,
                },
                Case {
                    name: "Carry",
                    input: 0x0000,
                    expected: 0xffff,
                },
            ], |case: &Case| {
                set(case.input, &mut self.cpu);
                self.exec_step();
                let actual = get(&self.cpu);

                assert_result!(HEX16, "program counter", 0x0001, self.cpu.regs().pc());
                assert_result!(HEX16, "result", case.expected, actual);
                self.assert_all_flags_unaffected("DEC (16-bits)");
            });
        }

        fn asset_behaves_like_add16<S, G>(&mut self, set: S, get: G)
        where S: Fn(u16, u16, &mut CPU), G: Fn(&CPU) -> u16 {
            struct Case {
                name: &'static str,
                a: u16,
                b: u16,
                expected: u16,
                expected_flags: fn(u8) -> u8,
            }
            table_test!(&[
                Case {
                    name: "Regular case",
                    a: 0x1245,
                    b: 0x1921,
                    expected: 0x2b66,
                    expected_flags: |f| flags_apply!(f, H:0 N:0 C:0),
                },
                Case {
                    name: "Half carry",
                    a: 0x1f45,
                    b: 0x1921,
                    expected: 0x3866,
                    expected_flags: |f| flags_apply!(f, H:1 N:0 C:0),
                },
                Case {
                    name: "Carry",
                    a: 0xff45,
                    b: 0x1921,
                    expected: 0x1866,
                    expected_flags: |f| flags_apply!(f, H:1 N:0 C:1),
                },
            ], |case: &Case| {
                set(case.a, case.b, &mut self.cpu);
                self.exec_step();
                assert_result!(HEX16, "program counter", 0x0001, self.cpu.regs().pc());
                let actual = get(&self.cpu);
                let expected_flags = (case.expected_flags)(self.cpu.regs().flags());
                let actual_flags = self.cpu.regs().flags();
                assert_result!(HEX16, "dest", case.expected, actual);
                assert_result!(BIN8, "flags", expected_flags, actual_flags);
            });
        }

        fn assert_sflag_if(&self, pre: &str, active: bool) {
            self.assert_flag_if(pre, active, "S", 0x80);
        }

        fn assert_zflag_if(&self, pre: &str, active: bool) {
            self.assert_flag_if(pre, active, "Z", 0x40);
        }

        fn assert_hflag_if(&self, pre: &str, active: bool) {
            self.assert_flag_if(pre, active, "H", 0x10);
        }

        fn assert_pvflag_if(&self, pre: &str, active: bool) {
            self.assert_flag_if(pre, active, "PV", 0x04);
        }

        fn assert_nflag_if(&self, pre: &str, active: bool) {
            self.assert_flag_if(pre, active, "N", 0x02);
        }

        fn assert_cflag_if(&self, pre: &str, active: bool) {
            self.assert_flag_if(pre, active, "C", 0x01);
        }

        fn assert_flag_if(&self, pre: &str, active: bool, name: &str, mask: u8) {
            let flags = self.cpu.regs().flags();
            if active {
                assert!(flags & mask != 0,
                    "{}: expected {} flag to be set in 0b{:08b}", pre, name, flags);
            } else {
                assert!(flags & mask == 0,
                    "{}: expected {} flag to be unset in 0b{:08b}", pre, name, flags);
            }
        }

        fn assert_sflag_unaffected(&self, pre: &str) {
            self.assert_flag_unaffected(pre, "S", 0x80);
        }

        fn assert_zflag_unaffected(&self, pre: &str) {
            self.assert_flag_unaffected(pre, "Z", 0x40);
        }

        fn assert_hflag_unaffected(&self, pre: &str) {
            self.assert_flag_unaffected(pre, "H", 0x10);
        }

        fn assert_pvflag_unaffected(&self, pre: &str) {
            self.assert_flag_unaffected(pre, "PV", 0x04);
        }

        fn assert_nflag_unaffected(&self, pre: &str) {
            self.assert_flag_unaffected(pre, "N", 0x02);
        }

        fn assert_cflag_unaffected(&self, pre: &str) {
            self.assert_flag_unaffected(pre, "C", 0x01);
        }

        fn assert_all_flags_unaffected(&self, pre: &str) {
            self.assert_sflag_unaffected(pre);
            self.assert_zflag_unaffected(pre);
            self.assert_hflag_unaffected(pre);
            self.assert_pvflag_unaffected(pre);
            self.assert_nflag_unaffected(pre);
            self.assert_cflag_unaffected(pre);
        }

        fn assert_flag_unaffected(&self, pre: &str, name: &str, mask: u8) {
            let flags = self.cpu.regs().flags();
            assert!(flags & mask == self.prev_flags & mask,
                "{}: expected {} flag to be unaffected in b{:08b} (previous flags were b{:08b}",
                pre, name, flags, self.prev_flags);
        }
    }
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

    #[bench]
    fn bench_exec_100_ld8(b: &mut Bencher) {
        exec_inst(b, &inst!(LD B, 0x42));
    }

    fn exec_inst(b: &mut Bencher, mut inst: &[u8]) {
        let mem = z80::MemoryBank::from_data(&mut inst).unwrap();
        let mut cpu = z80::CPU::new(z80::Options::default(), mem);
        b.iter(|| {
            for _ in 1..100 {
                test::black_box(exec_step(&mut cpu));
            }
        })
    }
}
