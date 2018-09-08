use byteorder::LittleEndian;

use bus::{Bus, ReadFromBytes};
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
        0xc3 => { ctx.exec_jp::<L16>();         10 },
        _ => unimplemented!("cannot execute illegal instruction with opcode 0x{:x}", opcode),
    }
}

/********************************************************/

trait Execute : Context + Sized {
    fn exec_add16<D: Src16 + Dest16, S: Src16>(&mut self) {
        let a = D::read_arg(self);
        let b = S::read_arg(self);

        let c = (a.data as u32) + (b.data as u32);
        D::write_arg(self, c as u16);
        self.regs_mut().inc_pc(1 + a.mem_bytes + b.mem_bytes);

        let flags = flags_apply!(self.regs().flags(), 
            C:[c>0xffff]
            H:[((a.data & 0x0fff) + (b.data & 0x0fff)) & 0x1000 != 0]
            N:0);
        self.regs_mut().set_flags(flags);
    }

    fn exec_dec8<D: Src8 + Dest8>(&mut self) {
        let fetch = D::read_arg(self);
        let mut flags = self.regs().flags();
        let result = self.alu().sub8_with_flags(fetch.data, 1, &mut flags);
        D::write_arg(self, result);
        self.regs_mut().inc_pc(1 + fetch.mem_bytes);
        self.regs_mut().set_flags(flags);
    }

    fn exec_dec16<D: Src16 + Dest16>(&mut self) {
        let fetch = D::read_arg(self);
        D::write_arg(self, fetch.data - 1);
        self.regs_mut().inc_pc(1 + fetch.mem_bytes);
    }

    fn exec_djnz(&mut self) -> bool {
        let b = self.alu().sub8(self.regs().b(), 1);
        self.regs_mut().set_b(b);
        if b > 0 {
            let s = self.read_from_pc(1);
            let pc = self.regs().pc();
            self.regs_mut().inc_pc(s as i8 as usize);
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
        let fetch = D::read_arg(self);
        let mut flags = self.regs().flags();
        let result = self.alu().add8_with_flags(fetch.data, 1, &mut flags);
        D::write_arg(self, result);
        self.regs_mut().inc_pc(1 + fetch.mem_bytes);
        self.regs_mut().set_flags(flags);
    }

    fn exec_inc16<D: Src16 + Dest16>(&mut self) {
        let fetch = D::read_arg(self);
        let result = (fetch.data as u32 + 1) as u16;
        D::write_arg(self, result);
        self.regs_mut().inc_pc(1 + fetch.mem_bytes);
    }

    fn exec_jp<S: Src16>(&mut self) {
        // TODO: cover more cases of jumps
        let s = S::read_arg(self);
        self.regs_mut().set_pc(s.data);
    }

    fn exec_jr<S: Src8>(&mut self) {
        // TODO: cover more cases of jumps
        let s = S::read_arg(self);
        self.regs_mut().inc_pc(s.data as i8 as usize);
    }

    fn exec_ld<D: Dest, S: Src>(&mut self)
    where D: Dest<Item=S::Item> {
        let fetch = S::read_arg(self);
        D::write_arg(self, fetch.data);
        self.regs_mut().inc_pc(1 + fetch.mem_bytes);
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


struct Fetch<T> {
    pub data: T,
    pub mem_bytes: usize,
}

trait Src {
    type Item;
    fn read_arg<C: Context>(ctx: &C) -> Fetch<Self::Item>;
}

trait Src8 : Src<Item=u8> {}
impl<T> Src8 for T where T: Src<Item=u8> {}

trait Src16 : Src<Item=u16> {}
impl<T> Src16 for T where T: Src<Item=u16> {}

trait Dest {
    type Item;
    fn write_arg<C: Context>(ctx: &mut C, val: Self::Item);
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
            fn read_arg<C: Context>(ctx: &C) -> Fetch<u8> {
                Fetch { data: ctx.regs().$r8r(), mem_bytes: 0 }
            }
        }

        impl Dest for $reg {
            type Item = u8;

            #[inline]
            fn write_arg<C: Context>(ctx: &mut C, val: u8) { ctx.regs_mut().$r8w(val) }
        }
    );
}

macro_rules! def_reg16_arg {
    ($reg:tt, $r16r:ident, $r16w:ident) => (
        struct $reg;

        impl Src for $reg {
            type Item = u16;

            #[inline]
            fn read_arg<C: Context>(ctx: &C) -> Fetch<u16> {
                Fetch { data: ctx.regs().$r16r(), mem_bytes: 0 }
            }
        }

        impl Dest for $reg {
            type Item = u16;

            #[inline]
            fn write_arg<C: Context>(ctx: &mut C, val: u16) { ctx.regs_mut().$r16w(val) }
        }
    );
}

macro_rules! def_indreg16_arg {
    ($reg:tt, $r16:ident) => (
        struct $reg;

        impl Src for $reg {
            type Item = u8;

            #[inline]
            fn read_arg<C: Context>(ctx: &C) -> Fetch<u8> {
                let addr = ctx.regs().$r16();
                let data = ctx.mem().read_from(addr);
                Fetch { data: data, mem_bytes: 0 }
            }
        }

        impl Dest for $reg {
            type Item = u8;

            #[inline]
            fn write_arg<C: Context>(ctx: &mut C, val: u8) {
                let addr = ctx.regs().$r16();
                ctx.mem_mut().write_to(addr, val);
            }
        }
    );
}

def_reg8_arg!(A, a, set_a);
def_reg8_arg!(B, b, set_b);
def_reg8_arg!(C, c, set_c);
def_reg8_arg!(D, d, set_d);
def_reg8_arg!(H, h, set_h);
def_reg8_arg!(L, l, set_l);

def_reg16_arg!(AF, af, set_af);
def_reg16_arg!(BC, bc, set_bc);
def_reg16_arg!(DE, de, set_de);
def_reg16_arg!(HL, hl, set_hl);

def_indreg16_arg!(IND_BC, bc);
def_indreg16_arg!(IND_DE, de);
def_indreg16_arg!(IND_HL, hl);

struct L8;
impl Src for L8 {
    type Item = u8;

    #[inline]
    fn read_arg<C: Context>(ctx: &C) -> Fetch<u8> {
        let pc = ctx.regs().pc();
        Fetch { data: ctx.mem().read_from(pc + 1), mem_bytes: 1 }
    }
}

struct L16;
impl Src for L16 {
    type Item = u16;

    #[inline]
    fn read_arg<C: Context>(ctx: &C) -> Fetch<u16> {
        let pc = ctx.regs().pc();
        Fetch { data: ctx.mem().read_word_from::<LittleEndian>(pc + 1), mem_bytes: 2 }
    }
}

/********************************************************/

#[cfg(test)]
mod test {
    use std::fmt;
    use std::io::Write;

    use rand;
    use rand::prelude::*;

    use cpu::z80;

    use super::*;

    #[test]
    fn test_exec_nop() {
        let mut test = ExecTest::for_inst(&inst!(NOP));
        test.exec_step();
        assert_eq!(0x0001, test.cpu.regs().pc());
        test.assert_all_flags_unaffected("nop");
    }

    #[test]
    fn test_exec_ld_bc_l16() {        
        let mut test = ExecTest::new();
        test.assert_behaves_like_ld(2, 
            |val, cpu| { Write::write(cpu.mem_mut(), &inst!(LD BC, val)).unwrap(); },
            |cpu| cpu.regs().bc(),
        );
    }

    #[test]
    fn test_exec_ld_indbc_a() {
        let mut test = ExecTest::for_inst(&inst!(LD (BC), A));
        test.assert_behaves_like_ld(0, 
            |val, cpu| {
                cpu.regs_mut().set_a(val);
                cpu.regs_mut().set_bc(0x1234);
            },
            |cpu| cpu.mem().read_from(0x1234),
        );
    }

    #[test]
    fn test_exec_inc_bc() {
        let mut test = ExecTest::for_inst(&inst!(INC BC));
        test.assert_behaves_like_inc16(
            |v, cpu| cpu.regs_mut().set_bc(v),
            |cpu| cpu.regs().bc(),
        );
    }

    #[test]
    fn test_exec_inc_b() {
        let mut test = ExecTest::for_inst(&inst!(INC B));
        test.assert_behaves_like_inc8(
            |v, cpu| cpu.regs_mut().set_b(v), 
            |cpu| cpu.regs().b(),
        );
    }

    #[test]
    fn test_exec_dec_b() {
        let mut test = ExecTest::for_inst(&inst!(DEC B));
        test.assert_behaves_like_dec8(
            |v, cpu| cpu.regs_mut().set_b(v), 
            |cpu| cpu.regs().b(),
        );        
    }

    #[test]
    fn test_exec_ld_b_l8() {
        let mut test = ExecTest::new();
        test.assert_behaves_like_ld(1, 
            |val, cpu| { Write::write(cpu.mem_mut(), &inst!(LD B, val)).unwrap(); },
            |cpu| cpu.regs().b(),
        );
    }

    #[test]
    fn test_exec_rlca() {
        let mut test = ExecTest::for_inst(&inst!(RLCA));
        for input in 0..=255 {
            test.cpu.regs_mut().set_a(input);
            test.exec_step();

            let pre = format!("RLCA b{:08b}", input);
            let carry = (input & 0b10000000) >> 7;
            let expected = (input << 1) | carry;
            let given = test.cpu.regs().a();
            assert_eq!(0x0001, test.cpu.regs().pc());
            assert_eq!(expected, given, "expected b{:08b} on {}, b{:08b} given", expected, pre, given);

            test.assert_sflag_unaffected(&pre);
            test.assert_zflag_unaffected(&pre);
            test.assert_hflag_if(&pre, false);
            test.assert_pvflag_unaffected(&pre);
            test.assert_nflag_if(&pre, false);
            test.assert_cflag_if(&pre, carry > 0);
        }
    }

    #[test]
    fn test_exec_exaf() {
        let mut test = ExecTest::for_inst(&inst!(EX AF, AF_));
        for _ in 0..=255 {
            let input = u16::sample();
            let input_ = u16::sample();;

            test.cpu.regs_mut().set_af(input);
            test.cpu.regs_mut().set_af_(input_);
            test.exec_step();

            let pre = "EX AF, AF'";
            let expected = input_;
            let expected_ = input;
            let given = test.cpu.regs().af();
            let given_ = test.cpu.regs().af_();
            assert_eq!(0x0001, test.cpu.regs().pc());
            assert_eq!(expected, given, "expected AF {} on {}, {} given", expected, pre, given);
            assert_eq!(expected_, given_, "expected AF' {} on {}, {} given", expected_, pre, given_);
        }
    }

    #[test]
    fn test_exec_add_hl_bc() {
        let mut test = ExecTest::for_inst(&inst!(ADD HL, BC));
        test.asset_behaves_like_add16(
            |a, b, cpu| {
                cpu.regs_mut().set_hl(a);
                cpu.regs_mut().set_bc(b);
            },
            |cpu| cpu.regs().hl(),
        );        
    }

    #[test]
    fn test_exec_ld_a_indbc() {
        let mut test = ExecTest::for_inst(&inst!(LD A, (BC)));
        test.assert_behaves_like_ld(0, 
            |val, cpu| { 
                cpu.mem_mut().write_to(0x1234, val);
                cpu.regs_mut().set_bc(0x1234);
            },
            |cpu| cpu.regs().a(),
        );
    }

    #[test]
    fn test_exec_dec_bc() {
        let mut test = ExecTest::for_inst(&inst!(DEC BC));
        test.assert_behaves_like_dec16(
            |v, cpu| cpu.regs_mut().set_bc(v), 
            |cpu| cpu.regs().bc(),
        );
    }

    #[test]
    fn test_exec_inc_c() {
        let mut test = ExecTest::for_inst(&inst!(INC C));
        test.assert_behaves_like_inc8(
            |v, cpu| cpu.regs_mut().set_c(v), 
            |cpu| cpu.regs().c(),
        );
    }

    #[test]
    fn test_exec_dec_c() {
        let mut test = ExecTest::for_inst(&inst!(DEC C));
        test.assert_behaves_like_dec8(
            |v, cpu| cpu.regs_mut().set_c(v), 
            |cpu| cpu.regs().c(),
        );        
    }

    #[test]
    fn test_exec_ld_c_l8() {
        let mut test = ExecTest::new();
        test.assert_behaves_like_ld(1, 
            |val, cpu| { Write::write(cpu.mem_mut(), &inst!(LD C, val)).unwrap(); },
            |cpu| cpu.regs().c(),
        );
    }

    #[test]
    fn test_exec_rrca() {
        let mut test = ExecTest::for_inst(&inst!(RRCA));
        for input in 0..=255 {
            test.cpu.regs_mut().set_a(input);
            test.exec_step();

            let pre = format!("RRCA b{:08b}", input);
            let carry = input & 0b00000001;
            let expected = (input >> 1) | (carry << 7);
            let given = test.cpu.regs().a();
            assert_eq!(0x0001, test.cpu.regs().pc());
            assert_eq!(expected, given, "expected b{:08b} on {}, b{:08b} given", expected, pre, given);

            test.assert_sflag_unaffected(&pre);
            test.assert_zflag_unaffected(&pre);
            test.assert_hflag_if(&pre, false);
            test.assert_pvflag_unaffected(&pre);
            test.assert_nflag_if(&pre, false);
            test.assert_cflag_if(&pre, carry > 0);
        }
    }

    #[test]
    fn test_exec_djnz_l8() {
        let mut test = ExecTest::new();
        for input in 0..=255 {
            let dest = u8::sample();
            test.cpu.mem_mut().write(&inst!(DJNZ dest)).unwrap();
            test.cpu.regs_mut().set_b(input);
            test.exec_step();

            let expected = test.cpu.alu().sub8(input, 1);
            let actual = test.cpu.regs().b();
            assert_eq!(expected, actual);

            let expected_pc = if expected > 0 {
                test.cpu.alu().add16(0, dest as i8 as u16)
            } else {
                2
            };

            let actual_pc = test.cpu.regs().pc();
            assert_eq!(expected_pc, actual_pc);

            test.assert_all_flags_unaffected("DJNZ");
        }
    }

    #[test]
    fn test_exec_ld_de_l16() {        
        let mut test = ExecTest::new();
        test.assert_behaves_like_ld(2, 
            |val, cpu| { Write::write(cpu.mem_mut(), &inst!(LD DE, val)).unwrap(); },
            |cpu| cpu.regs().de(),
        );
    }

    #[test]
    fn test_exec_ld_indde_a() {
        let mut test = ExecTest::for_inst(&inst!(LD (DE), A));
        test.assert_behaves_like_ld(0, 
            |val, cpu| {
                cpu.regs_mut().set_a(val);
                cpu.regs_mut().set_de(0x1234);
            },
            |cpu| cpu.mem().read_from(0x1234),
        );
    }

    #[test]
    fn test_exec_inc_de() {
        let mut test = ExecTest::for_inst(&inst!(INC DE));
        test.assert_behaves_like_inc16(
            |v, cpu| cpu.regs_mut().set_de(v),
            |cpu| cpu.regs().de(),
        );
    }

    #[test]
    fn test_exec_inc_d() {
        let mut test = ExecTest::for_inst(&inst!(INC D));
        test.assert_behaves_like_inc8(
            |v, cpu| cpu.regs_mut().set_d(v), 
            |cpu| cpu.regs().d(),
        );
    }

    #[test]
    fn test_exec_dec_d() {
        let mut test = ExecTest::for_inst(&inst!(DEC D));
        test.assert_behaves_like_dec8(
            |v, cpu| cpu.regs_mut().set_d(v), 
            |cpu| cpu.regs().d(),
        );        
    }

    #[test]
    fn test_exec_ld_d_l8() {
        let mut test = ExecTest::new();
        test.assert_behaves_like_ld(1, 
            |val, cpu| { Write::write(cpu.mem_mut(), &inst!(LD D, val)).unwrap(); },
            |cpu| cpu.regs().d(),
        );
    }

    #[test]
    fn test_exec_rla() {
        let mut test = ExecTest::for_inst(&inst!(RLA));
        for input in 0..=255 {
            let prev_carry = test.cpu.regs().flag_c();
            test.cpu.regs_mut().set_a(input);
            test.exec_step();

            let pre = format!("RLA b{:08b}", input);
            let carry = (input & 0b10000000) >> 7;
            let expected = (input << 1) | prev_carry;
            let given = test.cpu.regs().a();
            assert_eq!(0x0001, test.cpu.regs().pc());
            assert_eq!(expected, given, "expected b{:08b} on {}, b{:08b} given", expected, pre, given);

            test.assert_sflag_unaffected(&pre);
            test.assert_zflag_unaffected(&pre);
            test.assert_hflag_if(&pre, false);
            test.assert_pvflag_unaffected(&pre);
            test.assert_nflag_if(&pre, false);
            test.assert_cflag_if(&pre, carry > 0);
        }
    }

    #[test]
    fn test_exec_jr_l8() {
        let mut test = ExecTest::new();
        for input in 0..=255 {
            let dest = u8::sample();
            test.cpu.mem_mut().write(&inst!(JR dest)).unwrap();
            test.exec_step();

            let expected_pc = test.cpu.alu().add16(0, dest as i8 as u16);
            let actual_pc = test.cpu.regs().pc();
            assert_eq!(expected_pc, actual_pc);

            test.assert_all_flags_unaffected("JR");
        }
    }

    /*********************************************************/

    type CPU = z80::CPU<z80::MemoryBank>;

    struct ExecTest {
        pub cpu: CPU,
        prev_flags: u8,
    }

    trait Data : fmt::Display + fmt::Debug + Copy + PartialEq {
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
            let prev_flags = u8::sample();
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
            for _ in 0..=255 {
                let input = D::sample();
                set(input, &mut self.cpu);
                
                self.exec_step();

                let output = get(&self.cpu);
                let expected_pc = 1 + opsize as u16;
                let actual_pc = self.cpu.regs().pc();
                let flags = self.cpu.regs().flags();
                
                assert_eq!(expected_pc, actual_pc, "expected H{:04x} PC, but H{:04x} found", expected_pc, actual_pc);
                assert_eq!(input, output, "expected {} loaded value, but {} found", input, output);

                self.assert_all_flags_unaffected("LD");
            }
        }

        fn assert_behaves_like_inc8<S, G>(&mut self, set: S, get: G) 
        where S: Fn(u8, &mut CPU), G: Fn(&CPU) -> u8 {
            for input in 0..=255 {
                set(input, &mut self.cpu);
                self.exec_step();
                let expected = if input < 0xff { input + 1 } else { 0 };
                let actual = get(&self.cpu);
                
                assert_eq!(0x0001, self.cpu.regs().pc());
                assert_eq!(expected, actual);

                let flags = self.cpu.regs().flags();
                let pre = &format!("inc {}", input);

                // Check flags
                self.assert_sflag_if(&pre, actual & 0x80 != 0);
                self.assert_zflag_if(&pre, actual == 0);
                self.assert_hflag_if(&pre, input & 0x0f == 0x0f);
                self.assert_pvflag_if(&pre, input == 0x7f);
                self.assert_nflag_if(&pre, false);
                self.assert_cflag_if(&pre, input == 0xff);
            }
        }

        fn assert_behaves_like_inc16<S, G>(&mut self, set: S, get: G) 
        where S: Fn(u16, &mut CPU), G: Fn(&CPU) -> u16 {
            for _ in 0..=256 {
                let input = u16::sample();
                set(input, &mut self.cpu);
                self.exec_step();
                let expected = if input < 0xffff { input + 1 } else { 0 };
                let actual = get(&self.cpu);
                
                assert_eq!(0x0001, self.cpu.regs().pc());
                assert_eq!(expected, actual);

                self.assert_all_flags_unaffected("INC (16-bits)");
            }
        }

        fn assert_behaves_like_dec8<S, G>(&mut self, set: S, get: G) 
        where S: Fn(u8, &mut CPU), G: Fn(&CPU) -> u8 {
            for input in 0..=255 {
                set(input, &mut self.cpu);
                self.exec_step();
                let expected = if input > 0 { input - 1 } else { 0xff };
                let actual = get(&self.cpu);
                
                assert_eq!(0x0001, self.cpu.regs().pc());
                assert_eq!(expected, actual);

                let pre = &format!("dec {}", input);

                // Check flags
                self.assert_sflag_if(&pre, actual & 0x80 != 0);
                self.assert_zflag_if(&pre, actual == 0);
                self.assert_hflag_if(&pre, input & 0x0f == 0x00);
                self.assert_pvflag_if(&pre, input == 0x80);
                self.assert_nflag_if(&pre, true);
                self.assert_cflag_if(&pre, input == 0x00);
            }
        }

        fn assert_behaves_like_dec16<S, G>(&mut self, set: S, get: G) 
        where S: Fn(u16, &mut CPU), G: Fn(&CPU) -> u16 {
            for _ in 0..=256 {
                let input = u16::sample();
                set(input, &mut self.cpu);
                self.exec_step();
                let expected = if input > 0 { input - 1 } else { 0xffff };
                let actual = get(&self.cpu);
                
                assert_eq!(0x0001, self.cpu.regs().pc());
                assert_eq!(expected, actual);

                self.assert_all_flags_unaffected("DEC (16-bits)");
            }
        }

        fn asset_behaves_like_add16<S, G>(&mut self, set: S, get: G)
        where S: Fn(u16, u16, &mut CPU), G: Fn(&CPU) -> u16 {
            for _ in 0..=255 {
                let a = u16::sample();
                let b = u16::sample();
                let c = (a as u32) + (b as u32);
                set(a, b, &mut self.cpu);
                self.exec_step();
                let expected = c as u16;
                let actual = get(&self.cpu);
                
                assert_eq!(0x0001, self.cpu.regs().pc());
                assert_eq!(expected, actual);

                let pre = &format!("add b{:08b}, b{:08b}", a, b);

                // Check flags
                self.assert_sflag_unaffected(&pre);
                self.assert_zflag_unaffected(&pre);
                self.assert_hflag_if(&pre, ((a & 0xfff) + (b & 0xfff)) & 0x1000 != 0);
                self.assert_pvflag_unaffected(&pre);
                self.assert_nflag_if(&pre, false);
                self.assert_cflag_if(&pre, c > 0xffff);
            }
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
