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
}

pub fn exec_step<CTX: Context>(ctx: &mut CTX) -> Cycles {
    let pc = *ctx.regs().pc;
    let opcode = ctx.mem().read(pc);
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

        let flags = flags_apply!(self.regs().flags(), [C,H]:[c>0xffff] N:1);
        self.regs_mut().set_flags(flags);
    }

    fn exec_dec8<D: Src8 + Dest8>(&mut self) {
        let fetch = D::read_arg(self);
        D::write_arg(self, fetch.data - 1);
        self.regs_mut().inc_pc(1 + fetch.mem_bytes);
        let flags = flags_apply!(self.regs().flags(), N:0 PV:[fetch.data == 0]);
        self.regs_mut().set_flags(flags);
    }

    fn exec_dec16<D: Src16 + Dest16>(&mut self) {
        let fetch = D::read_arg(self);
        D::write_arg(self, fetch.data - 1);
        self.regs_mut().inc_pc(1 + fetch.mem_bytes);
    }

    fn exec_djnz<S: Src8>(&mut self) -> bool {
        let b = self.regs().bc.high() - 1;
        if b > 0 {
            let s = S::read_arg(self);
            let pc = *self.regs().pc;
            *self.regs_mut().pc = pc + (s.data as i8 as u16);
            true
        } else {
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
        let result = self.alu().add8(fetch.data, 1, &mut flags);
        D::write_arg(self, result);
        self.regs_mut().inc_pc(1 + fetch.mem_bytes);
        self.regs_mut().set_flags(flags);
    }

    fn exec_inc16<D: Src16 + Dest16>(&mut self) {
        let fetch = D::read_arg(self);
        D::write_arg(self, fetch.data + 1);
        self.regs_mut().inc_pc(1 + fetch.mem_bytes);
    }

    fn exec_jp<S: Src16>(&mut self) {
        // TODO: cover more cases of jumps
        let s = S::read_arg(self);
        *self.regs_mut().pc = s.data;
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

    fn exec_rlca(&mut self) {
        let orig = self.regs().af.high();
        let carry = orig >> 7;
        let dest = (orig << 1) | carry;
        self.regs_mut().af.set_high(dest);
        self.regs_mut().inc_pc(1);
        let flags = flags_apply!(self.regs().flags(), C:[carry > 0] H:0 N:0);
        self.regs_mut().set_flags(flags);
    }

    fn exec_rrca(&mut self) {
        let orig = self.regs().af.high();
        let carry = orig << 7;
        let dest = (orig >> 1) | carry;
        self.regs_mut().af.set_high(dest);
        self.regs_mut().inc_pc(1);
        let flags = flags_apply!(self.regs().flags(), C:[carry > 0] H:0 N:0);
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
    ($reg:tt, $r16:ident, $r8r:ident, $r8w:ident) => (
        struct $reg;

        impl Src for $reg {
            type Item = u8;

            #[inline]
            fn read_arg<C: Context>(ctx: &C) -> Fetch<u8> {
                Fetch { data: ctx.regs().$r16.$r8r(), mem_bytes: 0 }
            }
        }

        impl Dest for $reg {
            type Item = u8;

            #[inline]
            fn write_arg<C: Context>(ctx: &mut C, val: u8) { ctx.regs_mut().$r16.$r8w(val) }
        }
    );
}

macro_rules! def_reg16_arg {
    ($reg:tt, $r16:ident) => (
        struct $reg;

        impl Src for $reg {
            type Item = u16;

            #[inline]
            fn read_arg<C: Context>(ctx: &C) -> Fetch<u16> {
                Fetch { data: *ctx.regs().$r16, mem_bytes: 0 }
            }
        }

        impl Dest for $reg {
            type Item = u16;

            #[inline]
            fn write_arg<C: Context>(ctx: &mut C, val: u16) { *ctx.regs_mut().$r16 = val }
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
                let addr = *ctx.regs().$r16;
                let data = ctx.mem().read(addr);
                Fetch { data: data, mem_bytes: 0 }
            }
        }

        impl Dest for $reg {
            type Item = u8;

            #[inline]
            fn write_arg<C: Context>(ctx: &mut C, val: u8) {
                let addr = *ctx.regs().$r16;
                ctx.mem_mut().write(addr, val);
            }
        }
    );
}

def_reg8_arg!(A, af, high, set_high);
def_reg8_arg!(B, bc, high, set_high);
def_reg8_arg!(C, bc, low, set_low);
def_reg8_arg!(H, hl, high, set_high);
def_reg8_arg!(L, hl, low, set_low);

def_reg16_arg!(AF, af);
def_reg16_arg!(BC, bc);
def_reg16_arg!(HL, hl);

def_indreg16_arg!(IND_BC, bc);
def_indreg16_arg!(IND_HL, hl);

struct L8;
impl Src for L8 {
    type Item = u8;

    #[inline]
    fn read_arg<C: Context>(ctx: &C) -> Fetch<u8> {
        let pc = *ctx.regs().pc;
        Fetch { data: ctx.mem().read(pc + 1), mem_bytes: 1 }
    }
}

struct L16;
impl Src for L16 {
    type Item = u16;

    #[inline]
    fn read_arg<C: Context>(ctx: &C) -> Fetch<u16> {
        let pc = *ctx.regs().pc;
        Fetch { data: ctx.mem().read_word::<LittleEndian>(pc + 1), mem_bytes: 2 }
    }
}

/********************************************************/

mod test {
    use cpu::z80;

    use super::*;

    #[test]
    fn test_exec_nop() {
        let mut test = ExecTest::for_inst(&inst!(NOP));
        test.exec_step();
        assert_eq!(0x0001, *test.cpu.regs().pc);
        assert_eq!(0x00, test.cpu.regs().flags());
    }

    #[test]
    fn test_exec_ld_bc_l16() {
        let mut test = ExecTest::for_inst(&inst!(LD BC, 0x1234));
        test.exec_step();
        assert_eq!(0x0003, *test.cpu.regs().pc);
        assert_eq!(0x1234, *test.cpu.regs().bc);
        assert_eq!(0x00, test.cpu.regs().flags());
    }

    #[test]
    fn test_exec_ld_indbc_a() {
        let mut test = ExecTest::for_inst(&inst!(LD (BC), A));
        test.cpu.regs_mut().af.set_high(0x42);
        *test.cpu.regs_mut().bc = 0x1234;
        test.exec_step();
        assert_eq!(0x0001, *test.cpu.regs().pc);
        assert_eq!(0x42, test.cpu.mem().read_word::<LittleEndian>(0x1234));
        assert_eq!(0x00, test.cpu.regs().flags());
    }

    #[test]
    fn test_exec_inc_bc() {
        let mut test = ExecTest::for_inst(&inst!(INC BC));
        *test.cpu.regs_mut().bc = 0x1234;
        test.exec_step();
        assert_eq!(0x0001, *test.cpu.regs().pc);
        assert_eq!(0x1235, *test.cpu.regs().bc);
        assert_eq!(0x00, test.cpu.regs().flags());
    }

    #[test]
    fn test_exec_inc_b() {
        let mut test = ExecTest::for_inst(&inst!(INC B));
        test.cpu.regs_mut().bc.set_high(0x12);
        test.exec_step();
        assert_eq!(0x0001, *test.cpu.regs().pc);
        assert_eq!(0x13, test.cpu.regs().bc.high());
        assert_eq!(0b00000000, test.cpu.regs().flags());

        // Result is 0 (zero flag, carry flag, half-carry)
        test.test_flags(|cpu| { cpu.regs_mut().bc.set_high(0xff) }, 0b_0101_0001);

        // Result is 128 (sign flag, overflow, half-carry)
        test.test_flags(|cpu| { cpu.regs_mut().bc.set_high(0x7f) }, 0b_1001_0100);
    }

    /*
    #[test]
    fn test_exec_dec_b() {
        let mut test = ExecTest::for_inst(&inst!(DEC B));
        test.cpu.regs_mut().bc.set_high(0x12);
        test.exec_step();
        assert_eq!(0x0001, *test.cpu.regs().pc);
        assert_eq!(0x11, test.cpu.regs().bc.high());
        assert_eq!(0b00000010, test.cpu.regs().flags());

        // Result is 0 (zero flag)
        test.test_flags(|cpu| { cpu.regs_mut().bc.set_high(0x01) }, 0b01000000);

        // Result is -1 (sign flag, overflow)
        test.test_flags(|cpu| { cpu.regs_mut().bc.set_high(0x7f) }, 0b10000100);

        // Result is 16 (half-carry)
        test.test_flags(|cpu| { cpu.regs_mut().bc.set_high(0x0f) }, 0b00010000);
    }
    */

    struct ExecTest {
        pub cpu: z80::CPU<z80::MemoryBank>,
    }

    impl ExecTest {
        fn for_inst(mut inst: &[u8]) -> Self {
            let mem = z80::MemoryBank::from_data(&mut inst).unwrap();
            let cpu = z80::CPU::new(z80::Options::default(), mem);
            Self { cpu }
        }

        fn exec_step(&mut self) {
            *self.cpu.regs_mut().pc = 0x00;
            exec_step(&mut self.cpu);
        }

        fn test_flags<F: FnOnce(&mut z80::CPU<z80::MemoryBank>)>(&mut self, pre: F, expected_flags: u8) {
            pre(&mut self.cpu);
            self.exec_step();
            assert_eq!(expected_flags, self.cpu.regs().flags());
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
