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
        let mut flags = cpu_eval!($cpu, F);
        let c = $cpu.alu().add16_with_flags(a, b, &mut flags);
        cpu_eval!($cpu, $dst <- c as u16);
        cpu_eval!($cpu, PC ++<- 1 + op_size!($dst) + op_size!($src));
        cpu_eval!($cpu, F <- flags);
    });
    ($cpu:expr, AND $src:tt) => ({
        let a = cpu_eval!($cpu, A);
        let b = cpu_eval!($cpu, $src);
        let mut flags = 0;
        let c = $cpu.alu().bitwise_and(a, b, &mut flags);
        cpu_eval!($cpu, A <- c);
        cpu_eval!($cpu, PC ++<- 1 + op_size!($src));
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
        let mut flags = cpu_eval!($cpu, F);
        let a = $cpu.alu().bcd_adjust(cpu_eval!($cpu, A), &mut flags);
        cpu_eval!($cpu, A <- a);
        cpu_eval!($cpu, PC++);
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
        let b = $cpu.alu().sub8(cpu_eval!($cpu, B), 1);
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
        let result = $cpu.alu().add16(dest, 1);
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
        let dest = $cpu.alu().rotate_left(orig, &mut flags);
        cpu_eval!($cpu, A <- dest);
        cpu_eval!($cpu, PC++);
        cpu_eval!($cpu, F <- flags);
    });
    ($cpu:expr, RLCA) => ({
        let mut flags = cpu_eval!($cpu, F);
        let orig = cpu_eval!($cpu, A);
        let dest = $cpu.alu().rotate_left_with_carry(orig, &mut flags);
        cpu_eval!($cpu, A <- dest);
        cpu_eval!($cpu, PC++);
        cpu_eval!($cpu, F <- flags);
    });
    ($cpu:expr, RRA) => ({
        let mut flags = cpu_eval!($cpu, F);
        let orig = cpu_eval!($cpu, A);
        let dest = $cpu.alu().rotate_right(orig, &mut flags);
        cpu_eval!($cpu, A <- dest);
        cpu_eval!($cpu, PC++);
        cpu_eval!($cpu, F <- flags);
    });

    ($cpu:expr, RRCA) => ({
        let mut flags = cpu_eval!($cpu, F);
        let orig = cpu_eval!($cpu, A);
        let dest = $cpu.alu().rotate_right_with_carry(orig, &mut flags);
        cpu_eval!($cpu, A <- dest);
        cpu_eval!($cpu, PC++);
        cpu_eval!($cpu, F <- flags);
    });
    ($cpu:expr, SBC $dst:tt, $src:tt) => ({
        let a = cpu_eval!($cpu, $dst);
        let b = cpu_eval!($cpu, $src);
        let mut flags = cpu_eval!($cpu, F);
        let c = $cpu.alu().sbc8_with_flags(a, b, &mut flags);
        cpu_eval!($cpu, $dst <- c);
        cpu_eval!($cpu, PC ++<- 1 + op_size!($src));
        cpu_eval!($cpu, F <- flags);
    });
    ($cpu:expr, SCF) => ({
        let mut flags = cpu_eval!($cpu, F);
        flags = flags_apply!(flags, H:0 N:0 C:1);
        cpu_eval!($cpu, F <- flags);
    });
    ($cpu:expr, SUB $src:tt) => ({
        let a = cpu_eval!($cpu, A);
        let b = cpu_eval!($cpu, $src);
        let mut flags = 0;
        let c = $cpu.alu().sub8_with_flags(a, b, &mut flags);
        cpu_eval!($cpu, A <- c);
        cpu_eval!($cpu, PC ++<- 1 + op_size!($src));
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
        0x90 => { cpu_exec!(ctx, SUB B);            04 },
        0x91 => { cpu_exec!(ctx, SUB C);            04 },
        0x92 => { cpu_exec!(ctx, SUB D);            04 },
        0x93 => { cpu_exec!(ctx, SUB E);            04 },
        0x94 => { cpu_exec!(ctx, SUB H);            04 },
        0x95 => { cpu_exec!(ctx, SUB L);            04 },
        0x96 => { cpu_exec!(ctx, SUB (*HL));        07 },
        0x97 => { cpu_exec!(ctx, SUB A);            04 },
        0x98 => { cpu_exec!(ctx, SBC A, B);         04 },
        0x99 => { cpu_exec!(ctx, SBC A, C);         04 },
        0x9a => { cpu_exec!(ctx, SBC A, D);         04 },
        0x9b => { cpu_exec!(ctx, SBC A, E);         04 },
        0x9c => { cpu_exec!(ctx, SBC A, H);         04 },
        0x9d => { cpu_exec!(ctx, SBC A, L);         04 },
        0x9e => { cpu_exec!(ctx, SBC A, (*HL));     04 },
        0x9f => { cpu_exec!(ctx, SBC A, A);         04 },
        0xa0 => { cpu_exec!(ctx, AND B);            04 },
        0xa1 => { cpu_exec!(ctx, AND C);            04 },
        0xa2 => { cpu_exec!(ctx, AND D);            04 },
        0xa3 => { cpu_exec!(ctx, AND E);            04 },
        0xa4 => { cpu_exec!(ctx, AND H);            04 },
        0xa5 => { cpu_exec!(ctx, AND L);            04 },
        0xa6 => { cpu_exec!(ctx, AND (*HL));        07 },
        0xa7 => { cpu_exec!(ctx, AND A);            04 },

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

    decl_scenario!(exec_ld8, {
        macro_rules! decl_test_case {
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

        decl_test_case!(a_a, 1, A, A);
        decl_test_case!(a_b, 1, A, B);
        decl_test_case!(a_c, 1, A, C);
        decl_test_case!(a_d, 1, A, D);
        decl_test_case!(a_e, 1, A, E);
        decl_test_case!(a_h, 1, A, H);
        decl_test_case!(a_l, 1, A, L);
        decl_test_case!(b_a, 1, B, A);
        decl_test_case!(b_b, 1, B, B);
        decl_test_case!(b_c, 1, B, C);
        decl_test_case!(b_d, 1, B, D);
        decl_test_case!(b_e, 1, B, E);
        decl_test_case!(b_h, 1, B, H);
        decl_test_case!(b_l, 1, B, L);
        decl_test_case!(c_a, 1, C, A);
        decl_test_case!(c_b, 1, C, B);
        decl_test_case!(c_c, 1, C, C);
        decl_test_case!(c_d, 1, C, D);
        decl_test_case!(c_e, 1, C, E);
        decl_test_case!(c_h, 1, C, H);
        decl_test_case!(c_l, 1, C, L);
        decl_test_case!(d_a, 1, D, A);
        decl_test_case!(d_b, 1, D, B);
        decl_test_case!(d_c, 1, D, C);
        decl_test_case!(d_d, 1, D, D);
        decl_test_case!(d_e, 1, D, E);
        decl_test_case!(d_h, 1, D, H);
        decl_test_case!(d_l, 1, D, L);
        decl_test_case!(e_a, 1, E, A);
        decl_test_case!(e_b, 1, E, B);
        decl_test_case!(e_c, 1, E, C);
        decl_test_case!(e_d, 1, E, D);
        decl_test_case!(e_e, 1, E, E);
        decl_test_case!(e_h, 1, E, H);
        decl_test_case!(e_l, 1, E, L);
        decl_test_case!(h_a, 1, H, A);
        decl_test_case!(h_b, 1, H, B);
        decl_test_case!(h_c, 1, H, C);
        decl_test_case!(h_d, 1, H, D);
        decl_test_case!(h_e, 1, H, E);
        decl_test_case!(h_h, 1, H, H);
        decl_test_case!(h_l, 1, H, L);
        decl_test_case!(l_a, 1, L, A);
        decl_test_case!(l_b, 1, L, B);
        decl_test_case!(l_c, 1, L, C);
        decl_test_case!(l_d, 1, L, D);
        decl_test_case!(l_e, 1, L, E);
        decl_test_case!(l_h, 1, L, H);
        decl_test_case!(l_l, 1, L, L);

        decl_test_case!(indbc_a, 1, (*BC), A);
        decl_test_case!(indde_a, 1, (*DE), A);
        decl_test_case!(indhl_a, 1, (*HL), A);
        decl_test_case!(indhl_b, 1, (*HL), B);
        decl_test_case!(indhl_c, 1, (*HL), C);
        decl_test_case!(indhl_d, 1, (*HL), D);
        decl_test_case!(indhl_e, 1, (*HL), E);
        decl_test_case!(indhl_h, 1, (*HL), H);
        decl_test_case!(indhl_l, 1, (*HL), L);

        decl_test_case!(a_indbc, 1, A, (*BC));
        decl_test_case!(a_indde, 1, A, (*DE));
        decl_test_case!(a_indhl, 1, A, (*HL));
        decl_test_case!(b_indhl, 1, B, (*HL));
        decl_test_case!(c_indhl, 1, C, (*HL));
        decl_test_case!(d_indhl, 1, D, (*HL));
        decl_test_case!(e_indhl, 1, E, (*HL));
        decl_test_case!(h_indhl, 1, H, (*HL));
        decl_test_case!(l_indhl, 1, L, (*HL));

        decl_test_case!(indl16_a, 3, (*0x1234), A);

        decl_test_case!(a_l8, 2, A, n);
        decl_test_case!(b_l8, 2, B, n);
        decl_test_case!(c_l8, 2, C, n);
        decl_test_case!(d_l8, 2, D, n);
        decl_test_case!(e_l8, 2, E, n);
        decl_test_case!(h_l8, 2, H, n);
        decl_test_case!(l_l8, 2, L, n);

        decl_test_case!(indhl_l8, 2, (*HL), n);

        decl_test_case!(test_ld_a_indl16, 3, A, (*0x1234));
    });



    /*********************/
    /* 16-Bit Load Group */
    /*********************/

    decl_scenario!(exec_ld16, {
        macro_rules! decl_test_case {
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

        decl_test_case!(bc_l16, 3, BC, nn);
        decl_test_case!(de_l16, 3, DE, nn);
        decl_test_case!(hl_l16, 3, HL, nn);
        decl_test_case!(sp_l16, 3, SP, nn);

        decl_test_case!(indl16_hl, 3, (**nn), HL);

        decl_test_case!(hl_indl16, 3, HL, nn);
    });

    /**********************************************/
    /* Exchange, Block Transfer, and Search Group */
    /**********************************************/

    decl_test!(exec_exaf, {
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

    decl_scenario!(exec_add8, {
        macro_rules! decl_test_case {
            ($fname:ident, $dst:tt, $src:tt) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(ADD $dst, $src);
                    cpu_eval!(cpu, $dst <- 3);
                    cpu_eval!(cpu, $src <- 3);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r8!(cpu, $dst, 6);
                    assert_flags!(cpu, f0, (S:0 Z:0 H:0 PV:0 N:0 C:0));
                });
            };
        }

        decl_test_case!(a_a, A, A);
        decl_test_case!(a_b, A, B);
        decl_test_case!(a_c, A, C);
        decl_test_case!(a_d, A, D);
        decl_test_case!(a_e, A, E);
        decl_test_case!(a_h, A, H);
        decl_test_case!(a_indhl, A, (*HL));
        decl_test_case!(a_l, A, L);
    });

    decl_scenario!(exec_adc8, {
        macro_rules! decl_test_case {
            ($fname:ident, $dst:tt, $src:tt) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(ADC $dst, $src);
                    cpu_eval!(cpu, $dst <- 3);
                    cpu_eval!(cpu, $src <- 3);
                    cpu_eval!(cpu, F +<- (C:1));
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r8!(cpu, $dst, 7);
                    assert_flags!(cpu, f0, (S:0 Z:0 H:0 PV:0 N:0 C:0));
                });
            };
        }

        decl_test_case!(a_a, A, A);
        decl_test_case!(a_b, A, B);
        decl_test_case!(a_c, A, C);
        decl_test_case!(a_d, A, D);
        decl_test_case!(a_e, A, E);
        decl_test_case!(a_h, A, H);
        decl_test_case!(a_indhl, A, (*HL));
        decl_test_case!(a_l, A, L);
    });

    decl_scenario!(exec_sub, {
        macro_rules! decl_test_case {
            ($fname:ident, A) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(SUB A);
                    cpu_eval!(cpu, A <- 7);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r8!(cpu, A, 0);
                    assert_flags!(cpu, f0, (S:0 Z:1 H:0 PV:0 N:1 C:0));
                });
            };
            ($fname:ident, $src:tt) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(SUB $src);
                    cpu_eval!(cpu, A <- 7);
                    cpu_eval!(cpu, $src <- 3);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r8!(cpu, A, 4);
                    assert_flags!(cpu, f0, (S:0 Z:0 H:0 PV:0 N:1 C:0));
                });
            };
        }

        decl_test_case!(b, B);
        decl_test_case!(c, C);
        decl_test_case!(d, D);
        decl_test_case!(e, E);
        decl_test_case!(h, H);
        decl_test_case!(l, L);
        decl_test_case!(ind_hl, (*HL));
        decl_test_case!(a, A);
    });

    decl_scenario!(exec_sbc8, {
        macro_rules! decl_test_case {
            ($fname:ident, A, A) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(SBC A, A);
                    cpu_eval!(cpu, A <- 7);
                    cpu_eval!(cpu, F +<- (C:1));
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r8!(cpu, A, 0xff);
                    assert_flags!(cpu, f0, (S:1 Z:0 H:1 PV:0 N:1 C:1));
                });
            };
            ($fname:ident, $dst:tt, $src:tt) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(SBC $dst, $src);
                    cpu_eval!(cpu, $dst <- 7);
                    cpu_eval!(cpu, $src <- 3);
                    cpu_eval!(cpu, F +<- (C:1));
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r8!(cpu, $dst, 3);
                    assert_flags!(cpu, f0, (S:0 Z:0 H:0 PV:0 N:1 C:0));
                });
            };
        }

        decl_test_case!(a_b, A, B);
        decl_test_case!(a_c, A, C);
        decl_test_case!(a_d, A, D);
        decl_test_case!(a_e, A, E);
        decl_test_case!(a_h, A, H);
        decl_test_case!(a_l, A, L);
        decl_test_case!(a_hl, A, (*HL));
        decl_test_case!(a_a, A, A);
    });

    decl_scenario!(exec_and, {
        macro_rules! decl_test_case {
            ($fname:ident, A) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(AND A);
                    cpu_eval!(cpu, A <- 0b0101_1010);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r8!(cpu, A, 0b0101_1010);
                    assert_flags!(cpu, f0, (S:0 Z:0 H:1 PV:1 N:0 C:0));
                });
            };
            ($fname:ident, $src:tt) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(AND $src);
                    cpu_eval!(cpu, A <- 0b0101_1010);
                    cpu_eval!(cpu, $src <- 0b1010_1111);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r8!(cpu, A, 0b0000_1010);
                    assert_flags!(cpu, f0, (S:0 Z:0 H:1 PV:1 N:0 C:0));
                });
            };
        }

        decl_test_case!(b, B);
        decl_test_case!(c, C);
        decl_test_case!(d, D);
        decl_test_case!(e, E);
        decl_test_case!(h, H);
        decl_test_case!(l, L);
        decl_test_case!(ind_hl, (*HL));
        decl_test_case!(a, A);
    });

    decl_scenario!(exec_inc8, {
        macro_rules! decl_test_case {
            ($fname:ident, $dst:tt) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(INC $dst);
                    cpu_eval!(cpu, $dst <- 3);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r8!(cpu, $dst, 4);
                    assert_flags!(cpu, f0, (S:0 Z:0 H:0 PV:0 N:0));
                });
            };
        }

        decl_test_case!(a, A);
        decl_test_case!(c, C);
        decl_test_case!(d, D);
        decl_test_case!(e, E);
        decl_test_case!(h, H);
        decl_test_case!(l, L);
        decl_test_case!(indhl, (*HL));
    });

    decl_scenario!(exec_dec8, {
        macro_rules! decl_test_case {
            ($fname:ident, $dst:tt) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(DEC $dst);
                    cpu_eval!(cpu, $dst <- 3);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r8!(cpu, $dst, 2);
                    assert_flags!(cpu, f0, (S:0 Z:0 H:0 PV:0 N:1));
                });
            };
        }

        decl_test_case!(a, A);
        decl_test_case!(b, B);
        decl_test_case!(c, C);
        decl_test_case!(d, D);
        decl_test_case!(e, E);
        decl_test_case!(h, H);
        decl_test_case!(l, L);
        decl_test_case!(indhl, (*HL));
    });

    /*****************************************************/
    /* General-Purpose Arithmetic and CPU Control Groups */
    /*****************************************************/

    decl_test!(exec_cpl, {
        let mut cpu = cpu!(CPL);
        cpu_eval!(cpu, A <- 0x42);
        let f0 = exec_step!(&mut cpu);

        assert_r8!(cpu, A, 0xbd);
        assert_flags!(cpu, f0, (H:1 N:1));
    });

    decl_test!(exec_daa, {
        let mut cpu = cpu!(DAA);
        cpu_eval!(cpu, A <- 0xaa);
        cpu_eval!(cpu, F <- flags_apply!(0, N:0 H:0 C:0));
        let f0 = exec_step!(&mut cpu);
        assert_pc!(cpu, 0x0001);
        assert_r8!(cpu, A, 0x10);
        assert_flags!(cpu, f0, (N:0 H:1 C:1));
    });

    decl_test!(exec_nop, {
        let mut cpu = cpu!(NOP);
        let f0 = exec_step!(&mut cpu);
        assert_pc!(cpu, 0x0001);
        assert_flags!(cpu, f0, unaffected);
    });

    decl_test!(exec_scf, {
        let mut cpu = cpu!(SCF);
        let f0 = exec_step!(&mut cpu);
        assert_flags!(cpu, f0, (H:0 N:0 C:1));
    });

    decl_scenario!(exec_ccf, {
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

    decl_test!(exec_halt, {
        let mut cpu = cpu!(HALT);
        let f0 = exec_step!(&mut cpu);
        assert_pc!(cpu, 0x0000);
        assert_flags!(cpu, f0, unaffected);
    });

    /***************************/
    /* 16-Bit Arithmetic group */
    /***************************/

    decl_scenario!(exec_inc16, {
        macro_rules! decl_test_case {
            ($cname:ident, $dst:tt) => (
                decl_test!($cname, {
                    let mut cpu = cpu!(INC $dst);
                    cpu_eval!(cpu, $dst <- 0x1234);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r16!(cpu, $dst, 0x1235);
                    assert_flags!(cpu, f0, unaffected);
                });
            );
        }

        decl_test_case!(bc, BC);
        decl_test_case!(de, DE);
        decl_test_case!(hl, HL);
        decl_test_case!(sp, SP);
    });

    decl_scenario!(exec_dec16, {
        macro_rules! decl_test_case {
            ($cname:ident, $dst:tt) => (
                decl_test!($cname, {
                    let mut cpu = cpu!(DEC $dst);
                    cpu_eval!(cpu, $dst <- 0x1234);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r16!(cpu, $dst, 0x1233);
                    assert_flags!(cpu, f0, unaffected);
                });
            );
        }

        decl_test_case!(bc, BC);
        decl_test_case!(de, DE);
        decl_test_case!(hl, HL);
        decl_test_case!(sp, SP);
    });

    decl_scenario!(exec_add16, {
        macro_rules! decl_test_case {
            ($cname:ident, HL, HL) => (
                decl_test!($cname, {
                    let mut cpu = cpu!(ADD HL, HL);
                    cpu_eval!(cpu, HL <- 0x1050);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r16!(cpu, HL, 0x20a0);
                    assert_flags!(cpu, f0, (H:0 N:0 C:0));
                });
            );
            ($cname:ident, $dst:tt, $src:tt) => (
                decl_test!($cname, {
                    let mut cpu = cpu!(ADD $dst, $src);
                    cpu_eval!(cpu, $dst <- 0x1050);
                    cpu_eval!(cpu, $src <- 0x2310);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r16!(cpu, $dst, 0x3360);
                    assert_flags!(cpu, f0, (H:0 N:0 C:0));
                });
            );
        }

        decl_test_case!(hl_bc, HL, BC);
        decl_test_case!(hl_de, HL, DE);
        decl_test_case!(hl_sp, HL, SP);
        decl_test_case!(hl_hl, HL, HL);
    });

    /**************************/
    /* Rotate and Shift Group */
    /**************************/

    decl_test!(exec_rlca, {
        let mut cpu = cpu!(RLCA);
        cpu_eval!(cpu, A <- 0b0000_0010);
        let f0 = exec_step!(&mut cpu);

        assert_pc!(cpu, 0x0001);
        assert_r8!(cpu, A, 0b0000_0100);
        assert_flags!(cpu, f0, (H:0 N:0 C:0));
    });

    decl_test!(exec_rrca, {
        let mut cpu = cpu!(RRCA);
        cpu_eval!(cpu, A <- 0b0000_0010);
        let f0 = exec_step!(&mut cpu);

        assert_pc!(cpu, 0x0001);
        assert_r8!(cpu, A, 0b0000_0001);
        assert_flags!(cpu, f0, (H:0 N:0 C:0));
    });

    decl_test!(exec_rla, {
        let mut cpu = cpu!(RLA);
        cpu_eval!(cpu, F +<- (C:1));
        cpu_eval!(cpu, A <- 0b1000_1000);
        let f0 = exec_step!(&mut cpu);

        assert_pc!(cpu, 0x0001);
        assert_r8!(cpu, A, 0b0001_0001);
        assert_flags!(cpu, f0, (H:0 N:0 C:1));
    });

    decl_test!(exec_rra, {
        let mut cpu = cpu!(RRA);
        cpu_eval!(cpu, F +<- (C:1));
        cpu_eval!(cpu, A <- 0b0001_0001);
        let f0 = exec_step!(&mut cpu);

        assert_pc!(cpu, 0x0001);
        assert_r8!(cpu, A, 0b1000_1000);
        assert_flags!(cpu, f0, (H:0 N:0 C:1));
    });

    /**************/
    /* Jump Group */
    /**************/

    decl_scenario!(exec_djnz, {
        macro_rules! decl_test_case {
            ($cname:ident, $dst:expr, $input:expr, $output:expr, $pc:expr) => (
                decl_test!($cname, {
                    let mut cpu = cpu!(DJNZ $dst as u8);
                    cpu_eval!(cpu, B <- $input);
                    let f0 = exec_step!(&mut cpu);

                    assert_r8!(cpu, B, $output);
                    assert_pc!(cpu, $pc);
                    assert_flags!(cpu, f0, unaffected);
                });
            );
        }

        decl_test_case!(branch_forwards, 0x55, 10, 9, 0x0055);
        decl_test_case!(branch_backwards, -0x10i8, 10, 9, 0xfff0);
        decl_test_case!(no_branch, 0x55, 1, 0, 0x0002);
    });

    decl_scenario!(exec_jr_l8, {
        macro_rules! decl_test_case {
            ($cname:ident, $dst:expr, $pc:expr) => {
                decl_test!($cname, {
                    let mut cpu = cpu!(JR $dst as u8);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, $pc);
                    assert_flags!(cpu, f0, unaffected);
                });
            };
        }

        decl_test_case!(branch_forwards, 0x55, 0x0055);
        decl_test_case!(branch_backwards, -0x10i8, 0xfff0);
    });

    decl_scenario!(exec_jr_cond_l8, {
        macro_rules! decl_test_case {
            ($cname:ident, $cond:ident, $dst:expr, $input_flags:tt, $pc:expr) => {
                decl_test!($cname, {
                    let mut cpu = cpu!(JR $cond, $dst as u8);
                    cpu_eval!(cpu, F +<- $input_flags);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, $pc);
                    assert_flags!(cpu, f0, unaffected);
                });
            };
        }

        macro_rules! decl_test_suite {
            ($fname:ident, $cond:ident, $met_flags:tt, $unmet_flags:tt) => {
                decl_suite!($fname, {
                    decl_test_case!(branch_forwards, $cond, 0x55, $met_flags, 0x0055);
                    decl_test_case!(branch_backwards, $cond, -0x10i8, $met_flags, 0xfff0);
                    decl_test_case!(no_branch, $cond, 0x55, $unmet_flags, 0x0002);
                });
            };
        }

        decl_test_suite!(c, C, (C:1), (C:0));
        decl_test_suite!(nc, NC, (C:0), (C:1));
        decl_test_suite!(nz, NZ, (Z:0), (Z:1));
        decl_test_suite!(z, Z, (Z:1), (Z:0));
    });
}

#[cfg(all(feature = "nightly", test))]
mod bench {
    use test;
    use test::Bencher;

    use crate::cpu::z80;
    use super::*;

    #[bench]
    fn bench_exec_1000_cycles_of_add8(b: &mut Bencher) {
        exec_inst(b, &inst!(ADD A, B), 1000);
    }

    #[bench]
    fn bench_exec_1000_cycles_of_add16(b: &mut Bencher) {
        exec_inst(b, &inst!(ADD HL, BC), 1000);
    }

    #[bench]
    fn bench_exec_1000_cycles_of_and(b: &mut Bencher) {
        exec_inst(b, &inst!(AND B), 1000);
    }

    #[bench]
    fn bench_exec_1000_cycles_of_daa(b: &mut Bencher) {
        exec_inst(b, &inst!(DAA), 1000);
    }

    #[bench]
    fn bench_exec_1000_cycles_of_dec8(b: &mut Bencher) {
        exec_inst(b, &inst!(DEC B), 1000);
    }

    #[bench]
    fn bench_exec_1000_cycles_of_inc8(b: &mut Bencher) {
        exec_inst(b, &inst!(INC B), 1000);
    }

    #[bench]
    fn bench_exec_1000_cycles_of_jp_addr(b: &mut Bencher) {
        exec_inst(b, &inst!(JP 0x0000), 1000);
    }

    #[bench]
    fn bench_exec_1000_cycles_of_ld8(b: &mut Bencher) {
        exec_inst(b, &inst!(LD B, 0x42), 1000);
    }

    #[bench]
    fn bench_exec_1000_cycles_of_nops(b: &mut Bencher) {
        exec_inst(b, &inst!(NOP), 1000);
    }

    fn exec_inst(b: &mut Bencher, inst: &[u8], cycles: usize) {
        let mut mem = z80::MemoryBank::new();
        fill_instruction(&mut mem, inst, cycles);
        let mut cpu = z80::CPU::new(z80::Options::default(), mem);
        b.iter(|| {
            cpu_eval!(cpu, PC <- 0);
            let mut total_cycles = 0;
            while total_cycles < cycles {
                total_cycles += test::black_box(exec_step(&mut cpu));
            }
        })
    }

    fn fill_instruction(mem: &mut z80::MemoryBank, inst: &[u8], count: usize) {
        let mut addr = 0;
        for _ in 1..count {
            let mut src = inst;
            addr += mem.copy_to(addr, &mut src).unwrap() as u16;
        }
    }
}
