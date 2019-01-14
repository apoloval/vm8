use byteorder::LittleEndian;

use crate::bus::{Bus, ReadFromBytes, WriteFromBytes};
use crate::cpu::z80::alu::ALU;
use crate::cpu::z80::mem::MemoryBus;
use crate::cpu::z80::reg::Registers;

// Context trait defines a context where instructions are executed
pub trait Context {
    type Mem: MemoryBus;
    fn alu(&self) -> &ALU;
    fn regs(&self) -> &Registers;
    fn regs_mut(&mut self) -> &mut Registers;
    fn mem(&self) -> &Self::Mem;
    fn mem_mut(&mut self) -> &mut Self::Mem;

    fn read_from_pc(&self, offset: usize) -> u8 {
        let pc = cpu_eval!(self, PC);
        let pos = ((pc as usize) + offset) as u16;
        self.mem().read_from(pos)
    }
}

// Returns the size of the given operand encoded in the instruction
macro_rules! op_size {
    // Something relative to the PC occupies 1 byte
    (n) => { 1 };
    // Something relative to the PC read as word occupies 2 byte
    (nn) => { 2 };
    ((*nn)) => { 2 };
    ((**nn)) => { 2 };
    // Anything else occupies 0 bytes
    ($_:tt) => { 0 };
}

macro_rules! cpu_exec {
    ($cpu:expr, ADD8 $dst:tt, $src:tt) => ({
        let a = cpu_eval!($cpu, $dst);
        let b = cpu_eval!($cpu, $src);
        let mut flags = 0;
        let c = $cpu.alu().add8_with_flags(a, b, &mut flags);
        cpu_eval!($cpu, $dst <- c);
        cpu_eval!($cpu, PC ++<- 1 + op_size!($dst) + op_size!($src));
        cpu_eval!($cpu, F <- flags);
    });
    ($cpu:expr, ADC8 $dst:tt, $src:tt) => ({
        let a = cpu_eval!($cpu, $dst);
        let b = cpu_eval!($cpu, $src);
        let mut flags = cpu_eval!($cpu, F);
        let c = $cpu.alu().adc8_with_flags(a, b, &mut flags);
        cpu_eval!($cpu, $dst <- c);
        cpu_eval!($cpu, PC ++<- 1 + op_size!($dst) + op_size!($src));
        cpu_eval!($cpu, F <- flags);
    });
    ($cpu:expr, ADD16 $dst:tt, $src:tt) => ({
        let a = cpu_eval!($cpu, $dst);
        let b = cpu_eval!($cpu, $src);
        let c = (a as u32) + (b as u32);
        cpu_eval!($cpu, $dst <- c as u16);
        cpu_eval!($cpu, PC ++<- 1 + op_size!($dst) + op_size!($src));

        let flags = flags_apply!(cpu_eval!($cpu, F),
            C:[c>0xffff]
            H:[((a & 0x0fff) + (b & 0x0fff)) & 0x1000 != 0]
            N:0);
        cpu_eval!($cpu, F <- flags);
    });
    ($cpu:expr, CCF) => ({
        let mut flags = cpu_eval!($cpu, F);
        if flag!(flags, C) == 0 {
            flags = flags_apply!(flags, H:0 N:0 C:1);
        } else {
            flags = flags_apply!(flags, H:1 N:0 C:0);
        }
        cpu_eval!($cpu, F <- flags);
    });
    ($cpu:expr, CPL) => ({
        let a = cpu_eval!($cpu, A);
        cpu_eval!($cpu, A <- !a);

        let mut flags = cpu_eval!($cpu, F);
        flags = flags_apply!(flags, H:1 N:1);
        cpu_eval!($cpu, F <- flags);
    });
    ($cpu:expr, DAA) => ({
        let prev_a = cpu_eval!($cpu, A);
        let mut a = prev_a;
        let mut flags = cpu_eval!($cpu, F);
        if flag!(flags, N) == 0 {
            if flag!(flags, H) == 1 || a & 0x0f > 0x09 {
                a = $cpu.alu().add8(a, 0x06);
            }
            if flag!(flags, C) == 1 || a > 0x99 {
                a = $cpu.alu().add8(a, 0x60);
            }
        } else {
            if flag!(flags, H) == 1 || a & 0x0f > 0x09 {
                let (r, _) = $cpu.alu().sub8(a, 0x06);
                a = r;
            }
            if flag!(flags, C) == 1 || a > 0x99 {
                let (r, _) = $cpu.alu().sub8(a, 0x60);
                a = r;
            }
        }
        cpu_eval!($cpu, A <- a);
        cpu_eval!($cpu, PC++);

        flags = flags_apply!(flags,
            S:[a & 0x80 > 0]
            Z:[a == 0]
            H:[(a ^ prev_a) & 0x10 > 0]
            C:[flag!(flags, C) > 0 || prev_a > 0x99]
        );
        cpu_eval!($cpu, F <- flags);
    });
    ($cpu:expr, DEC8 $dst:tt) => ({
        let dest = cpu_eval!($cpu, $dst);
        let mut flags = cpu_eval!($cpu, F);
        let result = $cpu.alu().dec8_with_flags(dest, &mut flags);
        cpu_eval!($cpu, $dst <- result);
        cpu_eval!($cpu, PC ++<- 1 + op_size!($dst));
        cpu_eval!($cpu, F <- flags);
    });
    ($cpu:expr, DEC16 $dst:tt) => ({
        let dest = cpu_eval!($cpu, $dst);
        let result = $cpu.alu().sub16(dest, 1);
        cpu_eval!($cpu, $dst <- result);
        cpu_eval!($cpu, PC ++<- 1 + op_size!($dst));
    });
    ($cpu:expr, DJNZ) => ({
        let (b, _) = $cpu.alu().sub8(cpu_eval!($cpu, B), 1);
        cpu_eval!($cpu, B <- b);
        if b > 0 {
            let s = $cpu.read_from_pc(1);
            cpu_eval!($cpu, PC +<- s);
            true
        } else {
            cpu_eval!($cpu, PC ++<- 2);
            false
        }
    });
    ($cpu:expr, EXAF) => ({
        cpu_eval!($cpu, AF <-> AF_);
        cpu_eval!($cpu, PC++);
    });
    ($cpu:expr, HALT) => ({
    });
    ($cpu:expr, INC8 $dst:tt) => ({
        let dest = cpu_eval!($cpu, $dst);
        let mut flags = cpu_eval!($cpu, F);
        let result = $cpu.alu().inc8_with_flags(dest, &mut flags);
        cpu_eval!($cpu, $dst <- result);
        cpu_eval!($cpu, PC ++<- 1 + op_size!($dst));
        cpu_eval!($cpu, F <- flags);
    });
    ($cpu:expr, INC16 $dst:tt) => ({
        let dest = cpu_eval!($cpu, $dst);
        let (result, _) = $cpu.alu().add16(dest, 1);
        cpu_eval!($cpu, $dst <- result);
        cpu_eval!($cpu, PC ++<- 1 + op_size!($dst));
    });
    ($cpu:expr, JP $($dst:tt)+) => ({
        let dest = cpu_eval!($cpu, $($dst)+);
        cpu_eval!($cpu, PC <- dest);
    });
    ($cpu:expr, JR $dst:tt) => ({
        let dest = cpu_eval!($cpu, $dst);
        cpu_eval!($cpu, PC +<- dest);
    });
    ($cpu:expr, JR $f:tt, $dst:tt) => ({
        let flag = cpu_eval!($cpu, F[$f]);
        if flag == 1 {
            let dst = cpu_eval!($cpu, $dst);
            cpu_eval!($cpu, PC +<- dst);
            12
        } else {
            cpu_eval!($cpu, PC +<- 2);
            7
        }
    });
    ($cpu:expr, LD ($($dst:tt)+): u16, $src:tt) => ({
        let src = cpu_eval!($cpu, $src);
        cpu_eval!($cpu, ($($dst)+): u16 <- src);
        cpu_eval!($cpu, PC ++<- 1 + op_size!(($($dst)+): u16) + op_size!($src));
    });
    ($cpu:expr, LD $dst:tt, $($src:tt)+) => ({
        let src = cpu_eval!($cpu, $($src)+);
        cpu_eval!($cpu, $dst <- src);
        cpu_eval!($cpu, PC ++<- 1 + op_size!($dst) + op_size!($($src)+));
    });
    ($cpu:expr, NOP) => ({
        cpu_eval!($cpu, PC++);
    });
    ($cpu:expr, RLA) => ({
        let mut flags = cpu_eval!($cpu, F);
        let orig = cpu_eval!($cpu, A);
        let carry = flag!(cpu_eval!($cpu, F), C);
        let dest = $cpu.alu().rotate_left(orig, carry, &mut flags);
        cpu_eval!($cpu, A <- dest);
        cpu_eval!($cpu, PC++);
        cpu_eval!($cpu, F <- flags);
    });
    ($cpu:expr, RLCA) => ({
        let mut flags = cpu_eval!($cpu, F);
        let orig = cpu_eval!($cpu, A);
        let carry = (orig & 0x80) >> 7;
        let dest = $cpu.alu().rotate_left(orig, carry, &mut flags);
        cpu_eval!($cpu, A <- dest);
        cpu_eval!($cpu, PC++);
        cpu_eval!($cpu, F <- flags);
    });
    ($cpu:expr, RRA) => ({
        let mut flags = cpu_eval!($cpu, F);
        let orig = cpu_eval!($cpu, A);
        let carry = cpu_eval!($cpu, F[C]);
        let dest = $cpu.alu().rotate_right(orig, carry, &mut flags);
        cpu_eval!($cpu, A <- dest);
        cpu_eval!($cpu, PC++);
        cpu_eval!($cpu, F <- flags);
    });

    ($cpu:expr, RRCA) => ({
        let mut flags = cpu_eval!($cpu, F);
        let orig = cpu_eval!($cpu, A);
        let carry = orig & 0x01;
        let dest = $cpu.alu().rotate_right(orig, carry, &mut flags);
        cpu_eval!($cpu, A <- dest);
        cpu_eval!($cpu, PC++);
        cpu_eval!($cpu, F <- flags);
    });
    ($cpu:expr, SCF) => ({
        let mut flags = cpu_eval!($cpu, F);
        flags = flags_apply!(flags, H:0 N:0 C:1);
        cpu_eval!($cpu, F <- flags);
    });
}

/// Perform a execution step over the given context, returning the number of clock cyles required.
pub fn exec_step<CTX: Context>(ctx: &mut CTX) -> usize {
    let opcode = ctx.read_from_pc(0);
    match opcode {
        0x00 => { cpu_exec!(ctx, NOP);              04 },
        0x01 => { cpu_exec!(ctx, LD BC, nn);        10 },
        0x02 => { cpu_exec!(ctx, LD (*BC), A);      07 },
        0x03 => { cpu_exec!(ctx, INC16 BC);         06 },
        0x04 => { cpu_exec!(ctx, INC8 B);           04 },
        0x05 => { cpu_exec!(ctx, DEC8 B);           04 },
        0x06 => { cpu_exec!(ctx, LD B, n);          07 },
        0x07 => { cpu_exec!(ctx, RLCA);             04 },
        0x08 => { cpu_exec!(ctx, EXAF);             04 },
        0x09 => { cpu_exec!(ctx, ADD16 HL, BC);     11 },
        0x0a => { cpu_exec!(ctx, LD A, (*BC));      07 },
        0x0b => { cpu_exec!(ctx, DEC16 BC);         06 },
        0x0c => { cpu_exec!(ctx, INC8 C);           04 },
        0x0d => { cpu_exec!(ctx, DEC8 C);           04 },
        0x0e => { cpu_exec!(ctx, LD C, n);          07 },
        0x0f => { cpu_exec!(ctx, RRCA);             04 },
        0x10 => { if cpu_exec!(ctx, DJNZ) { 13 } else { 8 } },
        0x11 => { cpu_exec!(ctx, LD DE, nn);        10 },
        0x12 => { cpu_exec!(ctx, LD (*DE), A);      07 },
        0x13 => { cpu_exec!(ctx, INC16 DE);         06 },
        0x14 => { cpu_exec!(ctx, INC8 D);           04 },
        0x15 => { cpu_exec!(ctx, DEC8 D);           04 },
        0x16 => { cpu_exec!(ctx, LD D, n);          07 },
        0x17 => { cpu_exec!(ctx, RLA);              04 },
        0x18 => { cpu_exec!(ctx, JR n);             12 },
        0x19 => { cpu_exec!(ctx, ADD16 HL, DE);     11 },
        0x1a => { cpu_exec!(ctx, LD A, (*DE));      07 },
        0x1b => { cpu_exec!(ctx, DEC16 DE);         06 },
        0x1c => { cpu_exec!(ctx, INC8 E);           04 },
        0x1d => { cpu_exec!(ctx, DEC8 E);           04 },
        0x1e => { cpu_exec!(ctx, LD E, n);          07 },
        0x1f => { cpu_exec!(ctx, RRA);              04 },
        0x20 => { cpu_exec!(ctx, JR NZ, n) },
        0x21 => { cpu_exec!(ctx, LD HL, nn);        10 },
        0x22 => { cpu_exec!(ctx, LD (**nn), HL);    16 },
        0x23 => { cpu_exec!(ctx, INC16 HL);         06 },
        0x24 => { cpu_exec!(ctx, INC8 H);           04 },
        0x25 => { cpu_exec!(ctx, DEC8 H);           04 },
        0x26 => { cpu_exec!(ctx, LD H, n);          07 },
        0x27 => { cpu_exec!(ctx, DAA);              04 },
        0x28 => { cpu_exec!(ctx, JR Z, n) },
        0x29 => { cpu_exec!(ctx, ADD16 HL, HL);     11 },
        0x2a => { cpu_exec!(ctx, LD HL, (**nn));    16 },
        0x2b => { cpu_exec!(ctx, DEC16 HL);         06 },
        0x2c => { cpu_exec!(ctx, INC8 L);           04 },
        0x2d => { cpu_exec!(ctx, DEC8 L);           04 },
        0x2e => { cpu_exec!(ctx, LD L, n);          07 },
        0x2f => { cpu_exec!(ctx, CPL);              04 },
        0x30 => { cpu_exec!(ctx, JR NC, n) },
        0x31 => { cpu_exec!(ctx, LD SP, nn);        10 },
        0x32 => { cpu_exec!(ctx, LD (*nn), A);      13 },
        0x33 => { cpu_exec!(ctx, INC16 SP);         06 },
        0x34 => { cpu_exec!(ctx, INC8 (*HL));       11 },
        0x35 => { cpu_exec!(ctx, DEC8 (*HL));       11 },
        0x36 => { cpu_exec!(ctx, LD (*HL), n);      10 },
        0x37 => { cpu_exec!(ctx, SCF);              04 },
        0x38 => { cpu_exec!(ctx, JR C, n) },
        0x39 => { cpu_exec!(ctx, ADD16 HL, SP);     11 },
        0x3a => { cpu_exec!(ctx, LD A, (*nn));      13 },
        0x3b => { cpu_exec!(ctx, DEC16 SP);         06 },
        0x3c => { cpu_exec!(ctx, INC8 A);           04 },
        0x3d => { cpu_exec!(ctx, DEC8 A);           04 },
        0x3e => { cpu_exec!(ctx, LD A, n);          07 },
        0x3f => { cpu_exec!(ctx, CCF);              04 },
        0x40 => { cpu_exec!(ctx, LD B, B);          04 },
        0x41 => { cpu_exec!(ctx, LD B, C);          04 },
        0x42 => { cpu_exec!(ctx, LD B, D);          04 },
        0x43 => { cpu_exec!(ctx, LD B, E);          04 },
        0x44 => { cpu_exec!(ctx, LD B, H);          04 },
        0x45 => { cpu_exec!(ctx, LD B, L);          04 },
        0x46 => { cpu_exec!(ctx, LD B, (*HL));      07 },
        0x47 => { cpu_exec!(ctx, LD B, A);          04 },
        0x48 => { cpu_exec!(ctx, LD C, B);          04 },
        0x49 => { cpu_exec!(ctx, LD C, C);          04 },
        0x4a => { cpu_exec!(ctx, LD C, D);          04 },
        0x4b => { cpu_exec!(ctx, LD C, E);          04 },
        0x4c => { cpu_exec!(ctx, LD C, H);          04 },
        0x4d => { cpu_exec!(ctx, LD C, L);          04 },
        0x4e => { cpu_exec!(ctx, LD C, (*HL));      07 },
        0x4f => { cpu_exec!(ctx, LD C, A);          04 },
        0x50 => { cpu_exec!(ctx, LD D, B);          04 },
        0x51 => { cpu_exec!(ctx, LD D, C);          04 },
        0x52 => { cpu_exec!(ctx, LD D, D);          04 },
        0x53 => { cpu_exec!(ctx, LD D, E);          04 },
        0x54 => { cpu_exec!(ctx, LD D, H);          04 },
        0x55 => { cpu_exec!(ctx, LD D, L);          04 },
        0x56 => { cpu_exec!(ctx, LD D, (*HL));      07 },
        0x57 => { cpu_exec!(ctx, LD D, A);          04 },
        0x58 => { cpu_exec!(ctx, LD E, B);          04 },
        0x59 => { cpu_exec!(ctx, LD E, C);          04 },
        0x5a => { cpu_exec!(ctx, LD E, D);          04 },
        0x5b => { cpu_exec!(ctx, LD E, E);          04 },
        0x5c => { cpu_exec!(ctx, LD E, H);          04 },
        0x5d => { cpu_exec!(ctx, LD E, L);          04 },
        0x5e => { cpu_exec!(ctx, LD E, (*HL));      07 },
        0x5f => { cpu_exec!(ctx, LD E, A);          04 },
        0x60 => { cpu_exec!(ctx, LD H, B);          04 },
        0x61 => { cpu_exec!(ctx, LD H, C);          04 },
        0x62 => { cpu_exec!(ctx, LD H, D);          04 },
        0x63 => { cpu_exec!(ctx, LD H, E);          04 },
        0x64 => { cpu_exec!(ctx, LD H, H);          04 },
        0x65 => { cpu_exec!(ctx, LD H, L);          04 },
        0x66 => { cpu_exec!(ctx, LD H, (*HL));      07 },
        0x67 => { cpu_exec!(ctx, LD H, A);          04 },
        0x68 => { cpu_exec!(ctx, LD L, B);          04 },
        0x69 => { cpu_exec!(ctx, LD L, C);          04 },
        0x6a => { cpu_exec!(ctx, LD L, D);          04 },
        0x6b => { cpu_exec!(ctx, LD L, E);          04 },
        0x6c => { cpu_exec!(ctx, LD L, H);          04 },
        0x6d => { cpu_exec!(ctx, LD L, L);          04 },
        0x6e => { cpu_exec!(ctx, LD L, (*HL));      07 },
        0x6f => { cpu_exec!(ctx, LD L, A);          04 },
        0x70 => { cpu_exec!(ctx, LD (*HL), B);      07 },
        0x71 => { cpu_exec!(ctx, LD (*HL), C);      07 },
        0x72 => { cpu_exec!(ctx, LD (*HL), D);      07 },
        0x73 => { cpu_exec!(ctx, LD (*HL), E);      07 },
        0x74 => { cpu_exec!(ctx, LD (*HL), H);      07 },
        0x75 => { cpu_exec!(ctx, LD (*HL), L);      07 },
        0x76 => { cpu_exec!(ctx, HALT);             04 },
        0x77 => { cpu_exec!(ctx, LD (*HL), A);      07 },
        0x78 => { cpu_exec!(ctx, LD A, B);          04 },
        0x79 => { cpu_exec!(ctx, LD A, C);          04 },
        0x7a => { cpu_exec!(ctx, LD A, D);          04 },
        0x7b => { cpu_exec!(ctx, LD A, E);          04 },
        0x7c => { cpu_exec!(ctx, LD A, H);          04 },
        0x7d => { cpu_exec!(ctx, LD A, L);          04 },
        0x7e => { cpu_exec!(ctx, LD A, (*HL));      07 },
        0x7f => { cpu_exec!(ctx, LD A, A);          04 },
        0x80 => { cpu_exec!(ctx, ADD8 A, B);        04 },
        0x81 => { cpu_exec!(ctx, ADD8 A, C);        04 },
        0x82 => { cpu_exec!(ctx, ADD8 A, D);        04 },
        0x83 => { cpu_exec!(ctx, ADD8 A, E);        04 },
        0x84 => { cpu_exec!(ctx, ADD8 A, H);        04 },
        0x85 => { cpu_exec!(ctx, ADD8 A, L);        04 },
        0x86 => { cpu_exec!(ctx, ADD8 A, (*HL));    07 },
        0x87 => { cpu_exec!(ctx, ADD8 A, A);        04 },
        0x88 => { cpu_exec!(ctx, ADC8 A, B);        04 },
        0x89 => { cpu_exec!(ctx, ADC8 A, C);        04 },
        0x8a => { cpu_exec!(ctx, ADC8 A, D);        04 },
        0x8b => { cpu_exec!(ctx, ADC8 A, E);        04 },
        0x8c => { cpu_exec!(ctx, ADC8 A, H);        04 },
        0x8d => { cpu_exec!(ctx, ADC8 A, L);        04 },
        0x8e => { cpu_exec!(ctx, ADC8 A, (*HL));    07 },
        0x8f => { cpu_exec!(ctx, ADC8 A, A);        04 },

        0xc3 => { cpu_exec!(ctx, JP nn);            10 },
        _ => unimplemented!("cannot execute illegal instruction with opcode 0x{:x}", opcode),
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use crate::cpu::z80;
    use crate::testutil::Sample;

    use super::*;

    macro_rules! cpu {
        () => {
            {
                let mem = z80::MemoryBank::new();
                let mut cpu = z80::CPU::new(z80::Options::default(), mem);

                // Random flags, but do not set F5 and F3
                cpu_eval!(cpu, F <- u8::sample() & 0b11010111);

                // Pointer regs should reference valid memory addresses to be used as indirect
                // access arguments.
                cpu_eval!(cpu, BC <- 0x8000);
                cpu_eval!(cpu, DE <- 0x8010);
                cpu_eval!(cpu, HL <- 0x8020);

                cpu
            }
        };
        ($( $inst:tt )+) => {
            {
                let mut cpu = cpu!();
                Write::write(cpu.mem_mut(), &inst!($( $inst )+)).unwrap();
                cpu
            }
        };
    }

    macro_rules! exec_step {
        ($cpu:expr) => (
            {
                let f0 = cpu_eval!($cpu, F);
                cpu_eval!($cpu, PC <- 0x0000);
                exec_step($cpu);
                f0
            }
        )
    }

    /********************/
    /* 8-Bit Load Group */
    /********************/

    macro_rules! test_exec_ld8 {
        ($fname:ident, $pcinc:expr, $dst:tt, n) => {
            decl_test!($fname, {
                let input = u8::sample();
                let mut cpu = cpu!(LD $dst, input);
                let f0 = exec_step!(&mut cpu);
                assert_pc!(cpu, $pcinc);
                assert_r8!(cpu, $dst, input);
                assert_flags!(cpu, f0, unaffected);
            });
        };
        ($fname:ident, $pcinc:expr, $dst:tt, $src:tt) => {
            decl_test!($fname, {
                let input = u8::sample();
                let mut cpu = cpu!(LD $dst, $src);
                cpu_eval!(cpu, $src <- input);
                let f0 = exec_step!(&mut cpu);
                assert_pc!(cpu, $pcinc);
                assert_r8!(cpu, $dst, input);
                assert_flags!(cpu, f0, unaffected);
            });
        };
    }

    test_exec_ld8!(test_exec_ld8_a_a, 1, A, A);
    test_exec_ld8!(test_exec_ld8_a_b, 1, A, B);
    test_exec_ld8!(test_exec_ld8_a_c, 1, A, C);
    test_exec_ld8!(test_exec_ld8_a_d, 1, A, D);
    test_exec_ld8!(test_exec_ld8_a_e, 1, A, E);
    test_exec_ld8!(test_exec_ld8_a_h, 1, A, H);
    test_exec_ld8!(test_exec_ld8_a_l, 1, A, L);
    test_exec_ld8!(test_exec_ld8_b_a, 1, B, A);
    test_exec_ld8!(test_exec_ld8_b_b, 1, B, B);
    test_exec_ld8!(test_exec_ld8_b_c, 1, B, C);
    test_exec_ld8!(test_exec_ld8_b_d, 1, B, D);
    test_exec_ld8!(test_exec_ld8_b_e, 1, B, E);
    test_exec_ld8!(test_exec_ld8_b_h, 1, B, H);
    test_exec_ld8!(test_exec_ld8_b_l, 1, B, L);
    test_exec_ld8!(test_exec_ld8_c_a, 1, C, A);
    test_exec_ld8!(test_exec_ld8_c_b, 1, C, B);
    test_exec_ld8!(test_exec_ld8_c_c, 1, C, C);
    test_exec_ld8!(test_exec_ld8_c_d, 1, C, D);
    test_exec_ld8!(test_exec_ld8_c_e, 1, C, E);
    test_exec_ld8!(test_exec_ld8_c_h, 1, C, H);
    test_exec_ld8!(test_exec_ld8_c_l, 1, C, L);
    test_exec_ld8!(test_exec_ld8_d_a, 1, D, A);
    test_exec_ld8!(test_exec_ld8_d_b, 1, D, B);
    test_exec_ld8!(test_exec_ld8_d_c, 1, D, C);
    test_exec_ld8!(test_exec_ld8_d_d, 1, D, D);
    test_exec_ld8!(test_exec_ld8_d_e, 1, D, E);
    test_exec_ld8!(test_exec_ld8_d_h, 1, D, H);
    test_exec_ld8!(test_exec_ld8_d_l, 1, D, L);
    test_exec_ld8!(test_exec_ld8_e_a, 1, E, A);
    test_exec_ld8!(test_exec_ld8_e_b, 1, E, B);
    test_exec_ld8!(test_exec_ld8_e_c, 1, E, C);
    test_exec_ld8!(test_exec_ld8_e_d, 1, E, D);
    test_exec_ld8!(test_exec_ld8_e_e, 1, E, E);
    test_exec_ld8!(test_exec_ld8_e_h, 1, E, H);
    test_exec_ld8!(test_exec_ld8_e_l, 1, E, L);
    test_exec_ld8!(test_exec_ld8_h_a, 1, H, A);
    test_exec_ld8!(test_exec_ld8_h_b, 1, H, B);
    test_exec_ld8!(test_exec_ld8_h_c, 1, H, C);
    test_exec_ld8!(test_exec_ld8_h_d, 1, H, D);
    test_exec_ld8!(test_exec_ld8_h_e, 1, H, E);
    test_exec_ld8!(test_exec_ld8_h_h, 1, H, H);
    test_exec_ld8!(test_exec_ld8_h_l, 1, H, L);
    test_exec_ld8!(test_exec_ld8_l_a, 1, L, A);
    test_exec_ld8!(test_exec_ld8_l_b, 1, L, B);
    test_exec_ld8!(test_exec_ld8_l_c, 1, L, C);
    test_exec_ld8!(test_exec_ld8_l_d, 1, L, D);
    test_exec_ld8!(test_exec_ld8_l_e, 1, L, E);
    test_exec_ld8!(test_exec_ld8_l_h, 1, L, H);
    test_exec_ld8!(test_exec_ld8_l_l, 1, L, L);

    test_exec_ld8!(test_exec_ld8_indbc_a, 1, (*BC), A);
    test_exec_ld8!(test_exec_ld8_indde_a, 1, (*DE), A);
    test_exec_ld8!(test_exec_ld8_indhl_a, 1, (*HL), A);
    test_exec_ld8!(test_exec_ld8_indhl_b, 1, (*HL), B);
    test_exec_ld8!(test_exec_ld8_indhl_c, 1, (*HL), C);
    test_exec_ld8!(test_exec_ld8_indhl_d, 1, (*HL), D);
    test_exec_ld8!(test_exec_ld8_indhl_e, 1, (*HL), E);
    test_exec_ld8!(test_exec_ld8_indhl_h, 1, (*HL), H);
    test_exec_ld8!(test_exec_ld8_indhl_l, 1, (*HL), L);

    test_exec_ld8!(test_exec_ld8_a_indbc, 1, A, (*BC));
    test_exec_ld8!(test_exec_ld8_a_indde, 1, A, (*DE));
    test_exec_ld8!(test_exec_ld8_a_indhl, 1, A, (*HL));
    test_exec_ld8!(test_exec_ld8_b_indhl, 1, B, (*HL));
    test_exec_ld8!(test_exec_ld8_c_indhl, 1, C, (*HL));
    test_exec_ld8!(test_exec_ld8_d_indhl, 1, D, (*HL));
    test_exec_ld8!(test_exec_ld8_e_indhl, 1, E, (*HL));
    test_exec_ld8!(test_exec_ld8_h_indhl, 1, H, (*HL));
    test_exec_ld8!(test_exec_ld8_l_indhl, 1, L, (*HL));

    test_exec_ld8!(test_exec_ld8_indl16_a, 3, (*0x1234), A);

    test_exec_ld8!(test_exec_ld8_a_l8, 2, A, n);
    test_exec_ld8!(test_exec_ld8_b_l8, 2, B, n);
    test_exec_ld8!(test_exec_ld8_c_l8, 2, C, n);
    test_exec_ld8!(test_exec_ld8_d_l8, 2, D, n);
    test_exec_ld8!(test_exec_ld8_e_l8, 2, E, n);
    test_exec_ld8!(test_exec_ld8_h_l8, 2, H, n);
    test_exec_ld8!(test_exec_ld8_l_l8, 2, L, n);

    test_exec_ld8!(test_exec_ld8_indhl_l8, 2, (*HL), n);

    test_exec_ld8!(test_ld_a_indl16, 3, A, (*0x1234));


    /*********************/
    /* 16-Bit Load Group */
    /*********************/

    macro_rules! test_exec_ld16 {
        ($fname:ident, $pcinc:expr, (**nn), $src:tt) => {
            decl_test!($fname, {
                let input = u16::sample();
                let mut cpu = cpu!(LD (**0x4000), $src);
                cpu_eval!(cpu, $src <- input);
                let f0 = exec_step!(&mut cpu);
                assert_pc!(cpu, $pcinc);
                assert_r8!(cpu, (**0x4000), input);
                assert_flags!(cpu, f0, unaffected);
            });
        };
        ($fname:ident, $pcinc:expr, $dst:tt, nn) => {
            decl_test!($fname, {
                let input = u16::sample();
                let mut cpu = cpu!(LD $dst, input);
                cpu_eval!(cpu, $dst <- input);
                let f0 = exec_step!(&mut cpu);
                assert_pc!(cpu, $pcinc);
                assert_r8!(cpu, $dst, input);
                assert_flags!(cpu, f0, unaffected);
            });
        };
        ($fname:ident, $pcinc:expr, $dst:tt, $src:tt) => {
            decl_test!($fname, {
                let input = u16::sample();
                let mut cpu = cpu!(LD $dst, $src);
                cpu_eval!(cpu, $dst <- input);
                let f0 = exec_step!(&mut cpu);
                assert_pc!(cpu, $pcinc);
                assert_r8!(cpu, $dst, input);
                assert_flags!(cpu, f0, unaffected);
            });
        };
    }

    test_exec_ld16!(test_exec_ld16_bc_l16, 3, BC, nn);
    test_exec_ld16!(test_exec_ld16_de_l16, 3, DE, nn);
    test_exec_ld16!(test_exec_ld16_hl_l16, 3, HL, nn);
    test_exec_ld16!(test_exec_ld16_sp_l16, 3, SP, nn);

    test_exec_ld16!(test_exec_ld16_indl16_hl, 3, (**nn), HL);

    test_exec_ld16!(test_exec_ld16_hl_indl16, 3, HL, nn);

    /**********************************************/
    /* Exchange, Block Transfer, and Search Group */
    /**********************************************/

    decl_test!(test_exec_exaf, {
        let mut cpu = cpu!(EX AF, AF_);
        let af = cpu_eval!(cpu, AF <- u16::sample());
        let af_ = cpu_eval!(cpu, AF_ <- u16::sample());

        exec_step!(&mut cpu);
        assert_pc!(cpu, 0x0001);
        assert_r16!(cpu, AF, af_);
        assert_r16!(cpu, AF_, af);
    });

    /**************************/
    /* 8-Bit Arithmetic group */
    /**************************/

    macro_rules! test_exec_add8_case {
        ($dst:tt, $src:tt, $input:expr, $output:expr, $flags:tt) => ({
            let mut cpu = cpu!(ADD $dst, $src);
            cpu_eval!(cpu, $dst <- $input);
            cpu_eval!(cpu, $src <- $input);
            let f0 = exec_step!(&mut cpu);
            assert_pc!(cpu, 0x0001);
            assert_r8!(cpu, $dst, $output);
            assert_flags!(cpu, f0, $flags);
        });
    }

    macro_rules! test_exec_add8 {
        ($fname:ident, $pcinc:expr, $dst:tt, $src:tt) => {
            decl_suite!($fname, {
                decl_test!(regular_case, {
                    test_exec_add8_case!($dst, $src, 0x21, 0x42, (S:0 Z:0 H:0 PV:0 N:0 C:0));
                });
                decl_test!(overflow_plus_signed, {
                    test_exec_add8_case!($dst, $src, 0x51, 0xa2, (S:1 Z:0 H:0 PV:1 N:0 C:0));
                });
                decl_test!(half_carry, {
                    test_exec_add8_case!($dst, $src, 0x29, 0x52, (S:0 Z:0 H:1 PV:0 N:0 C:0));
                });
                decl_test!(zero, {
                    test_exec_add8_case!($dst, $src, 0, 0, (S:0 Z:1 H:0 PV:0 N:0 C:0));
                });
                decl_test!(carry, {
                    test_exec_add8_case!($dst, $src, 0x90, 0x20, (S:0 Z:0 H:0 PV:1 N:0 C:1));
                });
            });
        };
    }

    test_exec_add8!(test_exec_add_a_a, 1, A, A);
    test_exec_add8!(test_exec_add_a_a_2, 1, A, A);
    test_exec_add8!(test_exec_add_a_b, 1, A, B);
    test_exec_add8!(test_exec_add_a_c, 1, A, C);
    test_exec_add8!(test_exec_add_a_d, 1, A, D);
    test_exec_add8!(test_exec_add_a_e, 1, A, E);
    test_exec_add8!(test_exec_add_a_h, 1, A, H);
    test_exec_add8!(test_exec_add_a_l, 1, A, L);

    test_exec_add8!(test_exec_add_a_indhl, 1, A, (*HL));

    macro_rules! test_exec_adc8_case {
        ($dst:tt, $src:tt, $input:expr, $carry:expr, $output:expr, $flags:tt) => ({
            let mut cpu = cpu!(ADC $dst, $src);
            cpu_eval!(cpu, $dst <- $input);
            cpu_eval!(cpu, $src <- $input);
            cpu_eval!(cpu, F +<- (C:[$carry]));
            let f0 = exec_step!(&mut cpu);
            assert_pc!(cpu, 0x0001);
            assert_r8!(cpu, $dst, $output);
            assert_flags!(cpu, f0, $flags);
        });
    }

    macro_rules! test_exec_adc8 {
        ($fname:ident, $pcinc:expr, $dst:tt, $src:tt) => {
            decl_suite!($fname, {
                decl_test!(regular_case, {
                    test_exec_adc8_case!($dst, $src, 0x21, false, 0x42, (S:0 Z:0 H:0 PV:0 N:0 C:0));
                });
                decl_test!(regular_case_with_carry, {
                    test_exec_adc8_case!($dst, $src, 0x21, true, 0x43, (S:0 Z:0 H:0 PV:0 N:0 C:0));
                });
                decl_test!(overflow_plus_signed, {
                    test_exec_adc8_case!($dst, $src, 0x51, false, 0xa2, (S:1 Z:0 H:0 PV:1 N:0 C:0));
                });
                decl_test!(half_carry, {
                    test_exec_adc8_case!($dst, $src, 0x29, false, 0x52, (S:0 Z:0 H:1 PV:0 N:0 C:0));
                });
                decl_test!(zero, {
                    test_exec_adc8_case!($dst, $src, 0, false, 0, (S:0 Z:1 H:0 PV:0 N:0 C:0));
                });
                decl_test!(carry, {
                    test_exec_adc8_case!($dst, $src, 0x90, false, 0x20, (S:0 Z:0 H:0 PV:1 N:0 C:1));
                });
            });
        };
    }

    test_exec_adc8!(test_exec_adc_a_a, 1, A, A);
    test_exec_adc8!(test_exec_adc_a_b, 1, A, B);
    test_exec_adc8!(test_exec_adc_a_c, 1, A, C);
    test_exec_adc8!(test_exec_adc_a_d, 1, A, D);
    test_exec_adc8!(test_exec_adc_a_e, 1, A, E);
    test_exec_adc8!(test_exec_adc_a_h, 1, A, H);
    test_exec_adc8!(test_exec_adc_a_l, 1, A, L);

    test_exec_adc8!(test_exec_adc_a_indhl, 3, A, (*HL));

    macro_rules! test_exec_inc8_case {
        ($dst:tt, $input:expr, $output:expr, $flags:tt) => ({
            let mut cpu = cpu!(INC $dst);
            cpu_eval!(cpu, $dst <- $input);
            let f0 = exec_step!(&mut cpu);
            assert_pc!(cpu, 0x0001);
            assert_r8!(cpu, $dst, $output);
            assert_flags!(cpu, f0, $flags);
        });
    }

    macro_rules! test_exec_inc8 {
        ($fname:ident, $pcinc:expr, $dst:tt) => {
            decl_suite!($fname, {
                decl_test!(regular_case, {
                    test_exec_inc8_case!($dst, 0x01, 0x02, (S:0 Z:0 H:0 PV:0 N:0));
                });
                decl_test!(half_carry, {
                    test_exec_inc8_case!($dst, 0x0f, 0x10, (S:0 Z:0 H:1 PV:0 N:0));
                });
                decl_test!(overflow, {
                    test_exec_inc8_case!($dst, 0x7f, 0x80, (S:1 Z:0 H:1 PV:1 N:0));
                });
                decl_test!(carry, {
                    test_exec_inc8_case!($dst, 0xff, 0x00, (S:0 Z:1 H:1 PV:0 N:0));
                });
            });
        };
    }

    test_exec_inc8!(test_exec_inc_a, 1, A);
    test_exec_inc8!(test_exec_inc_c, 1, C);
    test_exec_inc8!(test_exec_inc_d, 1, D);
    test_exec_inc8!(test_exec_inc_e, 1, E);
    test_exec_inc8!(test_exec_inc_h, 1, H);
    test_exec_inc8!(test_exec_inc_l, 1, L);

    test_exec_inc8!(test_exec_inc_indhl, 1, (*HL));

    macro_rules! test_exec_dec8_case {
        ($dst:tt, $input:expr, $output:expr, $flags:tt) => ({
            let mut cpu = cpu!(DEC $dst);
            cpu_eval!(cpu, $dst <- $input);
            let f0 = exec_step!(&mut cpu);
            assert_pc!(cpu, 0x0001);
            assert_r8!(cpu, $dst, $output);
            assert_flags!(cpu, f0, $flags);
        });
    }

    macro_rules! test_exec_dec8 {
        ($fname:ident, $pcinc:expr, $dst:tt) => {
            decl_suite!($fname, {
                decl_test!(regular_case, {
                    test_exec_dec8_case!($dst, 0x02, 0x01, (S:0 Z:0 H:0 PV:0 N:1));
                });
                decl_test!(half_carry, {
                    test_exec_dec8_case!($dst, 0x10, 0x0f, (S:0 Z:0 H:1 PV:0 N:1));
                });
                decl_test!(overflow, {
                    test_exec_dec8_case!($dst, 0x80, 0x7f, (S:0 Z:0 H:1 PV:1 N:1));
                });
                decl_test!(zero, {
                    test_exec_dec8_case!($dst, 0x01, 0x00, (S:0 Z:1 H:0 PV:0 N:1));
                });
                decl_test!(no_carry, {
                    test_exec_dec8_case!($dst, 0x00, 0xff, (S:1 Z:0 H:1 PV:0 N:1));
                });
            });
        };
    }

    test_exec_dec8!(test_exec_dec_a, 1, A);
    test_exec_dec8!(test_exec_dec_b, 1, B);
    test_exec_dec8!(test_exec_dec_c, 1, C);
    test_exec_dec8!(test_exec_dec_d, 1, D);
    test_exec_dec8!(test_exec_dec_e, 1, E);
    test_exec_dec8!(test_exec_dec_h, 1, H);
    test_exec_dec8!(test_exec_dec_l, 1, L);

    test_exec_dec8!(test_exec_dec_indhl, 1, (*HL));

    /*****************************************************/
    /* General-Purpose Arithmetic and CPU Control Groups */
    /*****************************************************/

    decl_test!(test_exec_cpl, {
        let mut cpu = cpu!(CPL);
        cpu_eval!(cpu, A <- 0x42);
        let f0 = exec_step!(&mut cpu);

        assert_r8!(cpu, A, 0xbd);
        assert_flags!(cpu, f0, (H:1 N:1));
    });

    macro_rules! test_exec_daa_case {
        ($input:expr, $input_flags:expr, $output:expr, $output_flags:expr) => ({
            let mut cpu = cpu!(DAA);
            cpu_eval!(cpu, A <- $input);
            cpu_eval!(cpu, F <- $input_flags);
            exec_step!(&mut cpu);
            assert_pc!(cpu, 0x0001);
            assert_r8!(cpu, A, $output);
            assert_r8!(cpu, F, $output_flags);
        });
    }

    decl_suite!(test_exec_daa, {
        decl_test!(already_adjusted, {
            test_exec_daa_case!(
                0x42, flags_apply!(0, N:0 H:0 C:0),
                0x42, 0);
        });
        decl_test!(need_adjust_low_nibble_after_add, {
            test_exec_daa_case!(
                0x4d, flags_apply!(0, N:0 H:0 C:0),
                0x53, flags_apply!(0, N:0 H:1 C:0));
        });
        decl_test!(need_adjust_low_nibble_after_subtract, {
            test_exec_daa_case!(
                0x4d, flags_apply!(0, N:1 H:0 C:0),
                0x47, flags_apply!(0, N:1 H:0 C:0));
        });
        decl_test!(need_adjust_high_nibble_after_add, {
            test_exec_daa_case!(
                0xd4, flags_apply!(0, N:0 H:0 C:0),
                0x34, flags_apply!(0, N:0 H:0 C:1));
        });
        decl_test!(need_adjust_high_nibble_after_subtract, {
            test_exec_daa_case!(
                0xd4, flags_apply!(0, N:1 H:0 C:0),
                0x74, flags_apply!(0, N:1 H:0 C:1));
        });
    });

    decl_test!(test_exec_nop, {
        let mut cpu = cpu!(NOP);
        let f0 = exec_step!(&mut cpu);
        assert_pc!(cpu, 0x0001);
        assert_flags!(cpu, f0, unaffected);
    });

    decl_test!(test_exec_scf, {
        let mut cpu = cpu!(SCF);
        let f0 = exec_step!(&mut cpu);
        assert_flags!(cpu, f0, (H:0 N:0 C:1));
    });

    decl_suite!(test_exec_ccf, {
        decl_test!(flag_c_is_reset, {
            let mut cpu = cpu!(CCF);
            cpu_eval!(cpu, F <- 0);
            let f0 = exec_step!(&mut cpu);

            assert_flags!(cpu, f0, (H:0 N:0 C:1));
        });

        decl_test!(flag_c_is_set, {
            let mut cpu = cpu!(CCF);
            cpu_eval!(cpu, F <- 1);
            let f0 = exec_step!(&mut cpu);

            assert_flags!(cpu, f0, (H:1 N:0 C:0));
        });
    });

    decl_test!(test_exec_halt, {
        let mut cpu = cpu!(HALT);
        let f0 = exec_step!(&mut cpu);
        assert_pc!(cpu, 0x0000);
        assert_flags!(cpu, f0, unaffected);
    });

    /***************************/
    /* 16-Bit Arithmetic group */
    /***************************/

    macro_rules! test_inc16_case {
        ($dst:tt, $input:expr, $output:expr) => ({
            let mut cpu = cpu!(INC $dst);
            cpu_eval!(cpu, $dst <- $input);
            let f0 = exec_step!(&mut cpu);
            assert_pc!(cpu, 0x0001);
            assert_r16!(cpu, $dst, $output);
            assert_flags!(cpu, f0, unaffected);
        });
    }

    macro_rules! test_inc_reg16 {
        ($fname:ident, $dst:tt) => {
            decl_suite!($fname, {
                decl_test!(regular_case, { test_inc16_case!($dst, 0x0001, 0x0002) });
                decl_test!(carry, { test_inc16_case!($dst, 0xffff, 0x0000) });
            });
        }
    }

    test_inc_reg16!(test_exec_inc_bc, BC);
    test_inc_reg16!(test_exec_inc_de, DE);
    test_inc_reg16!(test_exec_inc_hl, HL);
    test_inc_reg16!(test_exec_inc_sp, SP);

    macro_rules! test_dec16_case {
        ($dst:tt, $input:expr, $output:expr) => ({
            let mut cpu = cpu!(DEC $dst);
            cpu_eval!(cpu, $dst <- $input);
            let f0 = exec_step!(&mut cpu);
            assert_pc!(cpu, 0x0001);
            assert_r16!(cpu, $dst, $output);
            assert_flags!(cpu, f0, unaffected);
        });
    }

    macro_rules! test_dec_reg16 {
        ($fname:ident, $dst:tt) => {
            decl_suite!($fname, {
                decl_test!(regular_case, { test_dec16_case!($dst, 0x0002, 0x0001) });
                decl_test!(carry, { test_dec16_case!($dst, 0x0000, 0xffff) });
            });
        }
    }

    test_dec_reg16!(test_exec_dec_bc, BC);
    test_dec_reg16!(test_exec_dec_de, DE);
    test_dec_reg16!(test_exec_dec_hl, HL);
    test_dec_reg16!(test_exec_dec_sp, SP);

    macro_rules! test_add16_case {
        ($dst:tt, $src:tt, $a:expr, $b:expr, $output:expr, $flags:tt) => ({
            let mut cpu = cpu!(ADD $dst, $src);
            cpu_eval!(cpu, $dst <- $a);
            cpu_eval!(cpu, $src <- $b);
            let f0 = exec_step!(&mut cpu);
            assert_pc!(cpu, 0x0001);
            assert_r16!(cpu, $dst, $output);
            assert_flags!(cpu, f0, $flags);
        });
    }

    macro_rules! test_add16 {
        ($fname:ident, $dst:tt, $src:tt) => {
            decl_suite!($fname, {
                decl_test!(regular_case, {
                    test_add16_case!($dst, $src, 0x1245, 0x1921, 0x2b66, (H:0 N:0 C:0));
                });
                decl_test!(half_carry, {
                    test_add16_case!($dst, $src, 0x1f45, 0x1921, 0x3866, (H:1 N:0 C:0));
                });
                decl_test!(carry, {
                    test_add16_case!($dst, $src, 0xff45, 0x1921, 0x1866, (H:1 N:0 C:1));
                });
            });
        };
        ($fname:ident, $dst:tt) => {
            decl_suite!($fname, {
                decl_test!(regular_case, {
                    test_add16_case!($dst, $dst, 0x1245, 0x1245, 0x248a, (H:0 N:0 C:0));
                });
                decl_test!(half_carry, {
                    test_add16_case!($dst, $dst, 0x1f45, 0x1f45, 0x3e8a, (H:1 N:0 C:0));
                });
                decl_test!(carry, {
                    test_add16_case!($dst, $dst, 0xff45, 0xff45, 0xfe8a, (H:1 N:0 C:1));
                });
            });
        };
    }

    test_add16!(test_exec_add_hl_bc, HL, BC);
    test_add16!(test_exec_add_hl_de, HL, DE);
    test_add16!(test_exec_add_hl_sp, HL, SP);
    test_add16!(test_exec_add_hl_hl, HL);

    /**************************/
    /* Rotate and Shift Group */
    /**************************/

    macro_rules! test_exec_rlca_case {
        ($input:expr, $output:expr, $flags:tt) => ({
            let mut cpu = cpu!(RLCA);
            cpu_eval!(cpu, A <- $input);
            let f0 = exec_step!(&mut cpu);

            assert_pc!(cpu, 0x0001);
            assert_r8!(cpu, A, $output);
            assert_flags!(cpu, f0, $flags);
        });
    }

    decl_suite!(test_exec_rlca, {
        decl_test!(no_carry, {
            test_exec_rlca_case!(0x12, 0x24, (H:0 N:0 C:0));
        });
        decl_test!(carry, {
            test_exec_rlca_case!(0xc8, 0x91, (H:0 N:0 C:1));
        });
    });

    macro_rules! test_exec_rrca_case {
        ($input:expr, $output:expr, $flags:tt) => ({
            let mut cpu = cpu!(RRCA);
            cpu_eval!(cpu, A <- $input);
            let f0 = exec_step!(&mut cpu);

            assert_pc!(cpu, 0x0001);
            assert_r8!(cpu, A, $output);
            assert_flags!(cpu, f0, $flags);
        });
    }

    decl_suite!(test_exec_rrca, {
        decl_test!(no_carry, {
            test_exec_rrca_case!(0x24, 0x12, (H:0 N:0 C:0));
        });
        decl_test!(carry, {
            test_exec_rrca_case!(0x91, 0xc8, (H:0 N:0 C:1));
        });
    });

    macro_rules! test_exec_rla_case {
        ($input:expr, $carry:expr, $output:expr, $flags:tt) => ({
            let mut cpu = cpu!(RLA);
            cpu_eval!(cpu, A <- $input);
            cpu_eval!(cpu, F +<- (C:[$carry]));
            let f0 = exec_step!(&mut cpu);

            assert_pc!(cpu, 0x0001);
            assert_r8!(cpu, A, $output);
            assert_flags!(cpu, f0, $flags);
        });
    }

    decl_suite!(test_exec_rla, {
        decl_test!(no_carry, {
            test_exec_rla_case!(0x12, false, 0x24, (H:0 N:0 C:0));
        });
        decl_test!(carry_in, {
            test_exec_rla_case!(0x12, true, 0x25, (H:0 N:0 C:0));
        });
        decl_test!(carry_out, {
            test_exec_rla_case!(0xc8, false, 0x90, (H:0 N:0 C:1));
        });
        decl_test!(carry_inout, {
            test_exec_rla_case!(0xc8, true, 0x91, (H:0 N:0 C:1));
        });
    });

    macro_rules! test_exec_rra_case {
        ($input:expr, $carry:expr, $output:expr, $flags:tt) => ({
            let mut cpu = cpu!(RRA);
            cpu_eval!(cpu, A <- $input);
            cpu_eval!(cpu, F +<- (C:[$carry]));
            let f0 = exec_step!(&mut cpu);

            assert_pc!(cpu, 0x0001);
            assert_r8!(cpu, A, $output);
            assert_flags!(cpu, f0, $flags);
        });
    }

    decl_suite!(test_exec_rra, {
        decl_test!(no_carry, {
            test_exec_rra_case!(0x24, false, 0x12, (H:0 N:0 C:0));
        });
        decl_test!(carry_in, {
            test_exec_rra_case!(0x24, true, 0x92, (H:0 N:0 C:0));
        });
        decl_test!(carry_out, {
            test_exec_rra_case!(0x91, false, 0x48, (H:0 N:0 C:1));
        });
        decl_test!(carry_inout, {
            test_exec_rra_case!(0x91, true, 0xc8, (H:0 N:0 C:1));
        });
    });

    /**************/
    /* Jump Group */
    /**************/

    macro_rules! test_exec_djnz_l8_case {
        ($dst:expr, $input:expr, $output:expr, $pc:expr) => ({
            let mut cpu = cpu!(DJNZ $dst as u8);
            cpu_eval!(cpu, B <- $input);
            let f0 = exec_step!(&mut cpu);

            assert_r8!(cpu, B, $output);
            assert_pc!(cpu, $pc);
            assert_flags!(cpu, f0, unaffected);
        });
    }

    decl_suite!(test_exec_djnz_l8, {
        decl_test!(branch_forwards, {
            test_exec_djnz_l8_case!(0x55, 10, 9, 0x0055);
        });
        decl_test!(branch_backwards, {
            test_exec_djnz_l8_case!(-0x10i8, 10, 9, 0xfff0);
        });
        decl_test!(no_branch, {
            test_exec_djnz_l8_case!(0x55, 1, 0, 0x0002);
        });
    });

    macro_rules! test_exec_jr_l8_case {
        ($dst:expr, $pc:expr) => ({
            let mut cpu = cpu!(JR $dst as u8);
            let f0 = exec_step!(&mut cpu);
            assert_pc!(cpu, $pc);
            assert_flags!(cpu, f0, unaffected);
        });
    }

    decl_suite!(test_exec_jr_l8, {
        decl_test!(branch_forwards, {
            test_exec_jr_l8_case!(0x55, 0x0055);
        });
        decl_test!(branch_backwards, {
            test_exec_jr_l8_case!(-0x10i8, 0xfff0);
        });
    });

    macro_rules! test_exec_jr_cond_l8_case {
        ($cond:ident, $dst:expr, $input_flags:tt, $pc:expr) => ({
            let mut cpu = cpu!(JR $cond, $dst as u8);
            cpu_eval!(cpu, F +<- $input_flags);
            let f0 = exec_step!(&mut cpu);
            assert_pc!(cpu, $pc);
            assert_flags!(cpu, f0, unaffected);
        });
    }

    macro_rules! test_jr_cond_l8 {
        ($fname:ident, $cond:ident, $met_flags:tt, $unmet_flags:tt) => {
            decl_suite!($fname, {
                decl_test!(branch_forwards, {
                    test_exec_jr_cond_l8_case!($cond, 0x55, $met_flags, 0x0055);
                });
                decl_test!(branch_backwards, {
                    test_exec_jr_cond_l8_case!($cond, -0x10i8, $met_flags, 0xfff0);
                });
                decl_test!(no_branch, {
                    test_exec_jr_cond_l8_case!($cond, 0x55, $unmet_flags, 0x0002);
                });
            });
        };
    }

    test_jr_cond_l8!(test_exec_jr_c_l8, C, (C:1), (C:0));
    test_jr_cond_l8!(test_exec_jr_nc_l8, NC, (C:0), (C:1));
    test_jr_cond_l8!(test_exec_jr_nz_l8, NZ, (Z:0), (Z:1));
    test_jr_cond_l8!(test_exec_jr_z_l8, Z, (Z:1), (Z:0));
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
