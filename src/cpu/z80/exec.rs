use byteorder::LittleEndian;

use crate::bus::Bus;
use crate::cpu::z80::alu::ALU;
use crate::cpu::z80::bus;
use crate::cpu::z80::reg::Registers;
use crate::mem::{ReadFromBytes, WriteFromBytes};

// Context trait defines a context where instructions are executed
pub trait Context {
    type Mem: bus::Memory;
    type IO: bus::IO;

    fn alu(&self) -> &ALU;
    fn regs(&self) -> &Registers;
    fn regs_mut(&mut self) -> &mut Registers;
    fn mem(&mut self) -> &mut Self::Mem;
    fn io(&mut self) -> &mut Self::IO;

    fn read_from_pc(&mut self, offset: usize) -> u8 {
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
    ($cpu:expr, CALL $dst:tt) => ({
        let dest = cpu_eval!($cpu, $dst);
        let pc = cpu_eval!($cpu, PC);
        let ret = pc + 1 + op_size!($dst);
        cpu_eval!($cpu, SP --<- 2);
        cpu_eval!($cpu, (**SP) <- ret);
        cpu_eval!($cpu, PC <- dest);
    });
    ($cpu:expr, CALL $cc:tt, $dst:tt) => ({
        let cond = cpu_eval!($cpu, F[$cc]);
        if cond > 0 {
            cpu_exec!($cpu, CALL $dst);
            17
        } else {
            cpu_eval!($cpu, PC +<- 1 + op_size!($dst));
            10
        }
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
    ($cpu:expr, CP $src:tt) => ({
        let a = cpu_eval!($cpu, A);
        let b = cpu_eval!($cpu, $src);
        let mut flags = 0;
        $cpu.alu().sub8_with_flags(a, b, &mut flags);
        cpu_eval!($cpu, PC ++<- 1 + op_size!($src));
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
    ($cpu:expr, EX $dst:tt, $src:tt) => ({
        let src = cpu_eval!($cpu, $src);
        let dst = cpu_eval!($cpu, $dst);
        cpu_eval!($cpu, $dst <- src);
        cpu_eval!($cpu, $src <- dst);
        cpu_eval!($cpu, PC++);
    });
    ($cpu:expr, EXAF) => ({
        cpu_eval!($cpu, AF <-> AF_);
        cpu_eval!($cpu, PC++);
    });
    ($cpu:expr, EXX) => ({
        cpu_eval!($cpu, BC <-> BC_);
        cpu_eval!($cpu, DE <-> DE_);
        cpu_eval!($cpu, HL <-> HL_);
        cpu_eval!($cpu, PC++);
    });
    ($cpu:expr, HALT) => ({
    });
    ($cpu:expr, IN $dst:tt, $src:tt) => ({
        let src = cpu_eval!($cpu, $src);
        cpu_eval!($cpu, $dst <- (!src));
        cpu_eval!($cpu, PC ++<- 1 + op_size!($dst) + op_size!($src));
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
    ($cpu:expr, JP $cc:tt, $dst:tt) => ({
        let cond = cpu_eval!($cpu, F[$cc]);
        if cond > 0 {
            cpu_exec!($cpu, JP $dst);
        } else {
            cpu_eval!($cpu, PC +<- 1 + op_size!($dst));
        }
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
            cpu_exec!($cpu, JR $dst);
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
    ($cpu:expr, OR $src:tt) => ({
        let a = cpu_eval!($cpu, A);
        let b = cpu_eval!($cpu, $src);
        let mut flags = 0;
        let c = $cpu.alu().bitwise_or(a, b, &mut flags);
        cpu_eval!($cpu, A <- c);
        cpu_eval!($cpu, PC ++<- 1 + op_size!($src));
        cpu_eval!($cpu, F <- flags);
    });
    ($cpu:expr, OUT $dst:tt, $src:tt) => ({
        let dst = cpu_eval!($cpu, $dst);

        cpu_eval!($cpu, (!dst) <- $src);
        cpu_eval!($cpu, PC ++<- 1 + op_size!($dst) + op_size!($src));
    });
    ($cpu:expr, POP $dst:tt) => ({
        let dest = cpu_eval!($cpu, (**SP));
        cpu_eval!($cpu, $dst <- dest);
        cpu_eval!($cpu, SP ++<- 2);
        cpu_eval!($cpu, PC++);
    });
    ($cpu:expr, PUSH $src:tt) => ({
        let src = cpu_eval!($cpu, $src);
        cpu_eval!($cpu, SP --<- 2);
        cpu_eval!($cpu, (**SP) <- src);
        cpu_eval!($cpu, PC++);
    });
    ($cpu:expr, RET) => ({
        let dest = cpu_eval!($cpu, (**SP));
        cpu_eval!($cpu, PC <- dest);
        cpu_eval!($cpu, SP ++<- 2);
    });
    ($cpu:expr, RET $cc:tt) => ({
        let cond = cpu_eval!($cpu, F[$cc]);
        if cond > 0 {
            cpu_exec!($cpu, RET);
            11
        } else {
            cpu_eval!($cpu, PC++);
            5
        }
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
    ($cpu:expr, RST $dst:tt) => ({
        let pc = cpu_eval!($cpu, PC);
        let ret = pc + 1 + op_size!($dst);
        cpu_eval!($cpu, SP --<- 2);
        cpu_eval!($cpu, (**SP) <- ret);
        cpu_eval!($cpu, PC <- $dst);
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
    ($cpu:expr, XOR $src:tt) => ({
        let a = cpu_eval!($cpu, A);
        let b = cpu_eval!($cpu, $src);
        let mut flags = 0;
        let c = $cpu.alu().bitwise_xor(a, b, &mut flags);
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
        0xa8 => { cpu_exec!(ctx, XOR B);            04 },
        0xa9 => { cpu_exec!(ctx, XOR C);            04 },
        0xaa => { cpu_exec!(ctx, XOR D);            04 },
        0xab => { cpu_exec!(ctx, XOR E);            04 },
        0xac => { cpu_exec!(ctx, XOR H);            04 },
        0xad => { cpu_exec!(ctx, XOR L);            04 },
        0xae => { cpu_exec!(ctx, XOR (*HL));        07 },
        0xaf => { cpu_exec!(ctx, XOR A);            04 },
        0xb0 => { cpu_exec!(ctx, OR B);             04 },
        0xb1 => { cpu_exec!(ctx, OR C);             04 },
        0xb2 => { cpu_exec!(ctx, OR D);             04 },
        0xb3 => { cpu_exec!(ctx, OR E);             04 },
        0xb4 => { cpu_exec!(ctx, OR H);             04 },
        0xb5 => { cpu_exec!(ctx, OR L);             04 },
        0xb6 => { cpu_exec!(ctx, OR (*HL));         07 },
        0xb7 => { cpu_exec!(ctx, OR A);             04 },
        0xb8 => { cpu_exec!(ctx, CP B);             04 },
        0xb9 => { cpu_exec!(ctx, CP C);             04 },
        0xba => { cpu_exec!(ctx, CP D);             04 },
        0xbb => { cpu_exec!(ctx, CP E);             04 },
        0xbc => { cpu_exec!(ctx, CP H);             04 },
        0xbd => { cpu_exec!(ctx, CP L);             04 },
        0xbe => { cpu_exec!(ctx, CP (*HL));         07 },
        0xbf => { cpu_exec!(ctx, CP A);             04 },
        0xc0 => { cpu_exec!(ctx, RET NZ) },
        0xc1 => { cpu_exec!(ctx, POP BC);           10 },
        0xc2 => { cpu_exec!(ctx, JP NZ, nn);        10 },
        0xc3 => { cpu_exec!(ctx, JP nn);            10 },
        0xc4 => { cpu_exec!(ctx, CALL NZ, nn) },
        0xc5 => { cpu_exec!(ctx, PUSH BC);          11 },
        0xc6 => { cpu_exec!(ctx, ADD8 A, n);        07 },
        0xc7 => { cpu_exec!(ctx, RST 0x00);         11 },
        0xc8 => { cpu_exec!(ctx, RET Z) },
        0xc9 => { cpu_exec!(ctx, RET);              10 },
        0xca => { cpu_exec!(ctx, JP Z, nn);         10 },
        0xcb => { unimplemented!("BITS table not yet implemented") },
        0xcc => { cpu_exec!(ctx, CALL Z, nn) },
        0xcd => { cpu_exec!(ctx, CALL nn);          17 },
        0xce => { cpu_exec!(ctx, ADC8 A, n);        07 },
        0xcf => { cpu_exec!(ctx, RST 0x08);         11 },
        0xd0 => { cpu_exec!(ctx, RET NC) },
        0xd1 => { cpu_exec!(ctx, POP DE);           10 },
        0xd2 => { cpu_exec!(ctx, JP NC, nn);        10 },
        0xd3 => { cpu_exec!(ctx, OUT n, A);         11 },
        0xd4 => { cpu_exec!(ctx, CALL NC, nn) },
        0xd5 => { cpu_exec!(ctx, PUSH DE);          11 },
        0xd6 => { cpu_exec!(ctx, SUB n);            07 },
        0xd7 => { cpu_exec!(ctx, RST 0x10);         11 },
        0xd8 => { cpu_exec!(ctx, RET C) },
        0xd9 => { cpu_exec!(ctx, EXX);              04 },
        0xda => { cpu_exec!(ctx, JP C, nn);         10 },
        0xdb => { cpu_exec!(ctx, IN A, n);         10 },
        0xdc => { cpu_exec!(ctx, CALL C, nn) },
        0xdd => { unimplemented!("IX table not yet implemented") },
        0xde => { cpu_exec!(ctx, SBC A, n);         07 },
        0xdf => { cpu_exec!(ctx, RST 0x18);         11 },
        0xe0 => { cpu_exec!(ctx, RET PO) },
        0xe1 => { cpu_exec!(ctx, POP HL);           10 },
        0xe2 => { cpu_exec!(ctx, JP PO, nn);        10 },
        0xe3 => { cpu_exec!(ctx, EX (**SP), HL);    19 },
        0xe4 => { cpu_exec!(ctx, CALL PO, nn) },
        0xe5 => { cpu_exec!(ctx, PUSH HL);          11 },
        0xe6 => { cpu_exec!(ctx, AND n);            07 },
        0xe7 => { cpu_exec!(ctx, RST 0x20);         11 },
        0xe8 => { cpu_exec!(ctx, RET PE) },
        0xe9 => { cpu_exec!(ctx, JP HL);            04 },
        0xea => { cpu_exec!(ctx, JP PE, nn);        10 },
        0xec => { cpu_exec!(ctx, CALL PE, nn) },
        0xee => { cpu_exec!(ctx, XOR n);            07 },
        0xef => { cpu_exec!(ctx, RST 0x28);         11 },
        0xf0 => { cpu_exec!(ctx, RET P) },
        0xf1 => { cpu_exec!(ctx, POP AF);           10 },
        0xf2 => { cpu_exec!(ctx, JP P, nn);         10 },
        0xf4 => { cpu_exec!(ctx, CALL P, nn) },
        0xf5 => { cpu_exec!(ctx, PUSH AF);          11 },
        0xf6 => { cpu_exec!(ctx, OR n);             07 },
        0xf7 => { cpu_exec!(ctx, RST 0x30);         11 },
        0xf8 => { cpu_exec!(ctx, RET M) },
        0xfa => { cpu_exec!(ctx, JP M, nn);         10 },
        0xfc => { cpu_exec!(ctx, CALL M, nn) },
        0xfe => { cpu_exec!(ctx, CP n);             07 },
        0xff => { cpu_exec!(ctx, RST 0x38);         11 },

        _ => unimplemented!("cannot execute illegal instruction with opcode 0x{:x}", opcode),
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use crate::cpu::z80;
    use crate::io;
    use crate::mem;
    use crate::testutil::Sample;

    use super::*;

    macro_rules! cpu {
        ($( $inst:tt )+) => {
            {
                let mut mem = Box::new(mem::MemoryBank::new());
                Write::write(&mut mem, &inst!($( $inst )+)).unwrap();

                let mut io = Box::new(io::Linear::new());
                io.bind(0x20, Box::new(io::Register::new()));

                let mut cpu = z80::CPU::new(z80::Options::default(), mem, io);

                // Random flags, but do not set F5 and F3
                cpu_eval!(cpu, F <- u8::sample() & 0b11010111);

                // Pointer regs should reference valid memory addresses to be used as indirect
                // access arguments.
                cpu_eval!(cpu, BC <- 0x4000);
                cpu_eval!(cpu, DE <- 0x4010);
                cpu_eval!(cpu, HL <- 0x4020);

                // Configure the stack
                cpu_eval!(cpu, SP <- 0x8000);

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

    decl_scenario!(exec_pop, {
        macro_rules! decl_test_case {
            ($cname:ident, AF) => {
                decl_test!($cname, {
                    let mut cpu = cpu!(POP AF);
                    cpu_eval!(cpu, SP <- 0x5000);
                    cpu_eval!(cpu, (**0x5000) <- 0x1234);

                    exec_step!(&mut cpu);

                    assert_pc!(cpu, 0x0001);
                    assert_r16!(cpu, AF, 0x1234);
                    assert_r16!(cpu, SP, 0x5002);
                });
            };
            ($cname:ident, $dst:tt) => {
                decl_test!($cname, {
                    let mut cpu = cpu!(POP $dst);
                    cpu_eval!(cpu, SP <- 0x5000);
                    cpu_eval!(cpu, (**0x5000) <- 0x1234);

                    let f0 = exec_step!(&mut cpu);

                    assert_pc!(cpu, 0x0001);
                    assert_r16!(cpu, $dst, 0x1234);
                    assert_r16!(cpu, SP, 0x5002);
                    assert_flags!(cpu, f0, unaffected);
                });
            };
        }

        decl_test_case!(bc, BC);
        decl_test_case!(de, DE);
        decl_test_case!(hl, HL);
        decl_test_case!(af, AF);
    });

    decl_scenario!(exec_push, {
        macro_rules! decl_test_case {
            ($cname:ident, $dst:tt) => {
                decl_test!($cname, {
                    let mut cpu = cpu!(PUSH $dst);
                    cpu_eval!(cpu, $dst <- 0x1234);
                    cpu_eval!(cpu, SP <- 0x5000);

                    let f0 = exec_step!(&mut cpu);

                    assert_pc!(cpu, 0x0001);
                    assert_r16!(cpu, (**0x4ffe), 0x1234);
                    assert_r16!(cpu, SP, 0x4ffe);
                    assert_flags!(cpu, f0, unaffected);
                });
            };
        }

        decl_test_case!(bc, BC);
        decl_test_case!(de, DE);
        decl_test_case!(hl, HL);
        decl_test_case!(af, AF);
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

    decl_test!(exec_exx, {
        let mut cpu = cpu!(EXX);
        cpu_eval!(cpu, BC <- 0x1000);
        cpu_eval!(cpu, DE <- 0x2000);
        cpu_eval!(cpu, HL <- 0x3000);
        cpu_eval!(cpu, BC_ <- 0x0010);
        cpu_eval!(cpu, DE_ <- 0x0020);
        cpu_eval!(cpu, HL_ <- 0x0030);

        let f0 = exec_step!(&mut cpu);

        assert_pc!(cpu, 0x0001);
        assert_r16!(cpu, BC, 0x0010);
        assert_r16!(cpu, DE, 0x0020);
        assert_r16!(cpu, HL, 0x0030);
        assert_r16!(cpu, BC_, 0x1000);
        assert_r16!(cpu, DE_, 0x2000);
        assert_r16!(cpu, HL_, 0x3000);
        assert_flags!(cpu, f0, unaffected);
    });

    decl_test!(exec_ex_ind_sp_hl, {
        let mut cpu = cpu!(EX (SP), HL);
        cpu_eval!(cpu, HL <- 0x1234);
        cpu_eval!(cpu, (**SP) <- 0xabcd);

        exec_step!(&mut cpu);

        assert_pc!(cpu, 0x0001);
        assert_r16!(cpu, HL, 0xabcd);
        assert_mem16!(cpu, SP, 0x1234);
    });

    /**************************/
    /* 8-Bit Arithmetic group */
    /**************************/

    decl_scenario!(exec_add8, {
        macro_rules! decl_test_case {
            ($fname:ident, $dst:tt, n) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(ADD $dst, 3);
                    cpu_eval!(cpu, $dst <- 3);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0002);
                    assert_r8!(cpu, $dst, 6);
                    assert_flags!(cpu, f0, (S:0 Z:0 H:0 PV:0 N:0 C:0));
                });
            };
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
        decl_test_case!(a_n, A, n);
    });

    decl_scenario!(exec_adc8, {
        macro_rules! decl_test_case {
            ($fname:ident, $dst:tt, n) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(ADC $dst, 3);
                    cpu_eval!(cpu, $dst <- 3);
                    cpu_eval!(cpu, F +<- (C:1));
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0002);
                    assert_r8!(cpu, $dst, 7);
                    assert_flags!(cpu, f0, (S:0 Z:0 H:0 PV:0 N:0 C:0));
                });
            };
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
        decl_test_case!(a_n, A, n);
    });

    decl_scenario!(exec_sub, {
        macro_rules! decl_test_case {
            ($fname:ident, n) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(SUB 3);
                    cpu_eval!(cpu, A <- 7);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0002);
                    assert_r8!(cpu, A, 4);
                    assert_flags!(cpu, f0, (S:0 Z:0 H:0 PV:0 N:1 C:0));
                });
            };
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
        decl_test_case!(n, n);
    });

    decl_scenario!(exec_sbc8, {
        macro_rules! decl_test_case {
            ($fname:ident, $dst:tt, n) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(SBC $dst, 3);
                    cpu_eval!(cpu, $dst <- 7);
                    cpu_eval!(cpu, F +<- (C:1));
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0002);
                    assert_r8!(cpu, $dst, 3);
                    assert_flags!(cpu, f0, (S:0 Z:0 H:0 PV:0 N:1 C:0));
                });
            };
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
        decl_test_case!(a_n, A, n);
    });

    decl_scenario!(exec_and, {
        macro_rules! decl_test_case {
            ($fname:ident, n) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(AND 0b1010_1111);
                    cpu_eval!(cpu, A <- 0b0101_1010);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0002);
                    assert_r8!(cpu, A, 0b0000_1010);
                    assert_flags!(cpu, f0, (S:0 Z:0 H:1 PV:1 N:0 C:0));
                });
            };
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

    decl_scenario!(exec_or, {
        macro_rules! decl_test_case {
            ($fname:ident, n) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(OR 0b1010_1111);
                    cpu_eval!(cpu, A <- 0b0101_1010);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r8!(cpu, A, 0b1111_1111);
                    assert_flags!(cpu, f0, (S:0 Z:0 H:0 PV:1 N:0 C:0));
                });
            };
            ($fname:ident, A) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(OR A);
                    cpu_eval!(cpu, A <- 0b0101_1010);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r8!(cpu, A, 0b0101_1010);
                    assert_flags!(cpu, f0, (S:0 Z:0 H:0 PV:1 N:0 C:0));
                });
            };
            ($fname:ident, $src:tt) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(OR $src);
                    cpu_eval!(cpu, A <- 0b0101_1010);
                    cpu_eval!(cpu, $src <- 0b1010_1111);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r8!(cpu, A, 0b1111_1111);
                    assert_flags!(cpu, f0, (S:1 Z:0 H:0 PV:1 N:0 C:0));
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

    decl_scenario!(exec_xor, {
        macro_rules! decl_test_case {
            ($fname:ident, n) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(XOR 0b1010_1111);
                    cpu_eval!(cpu, A <- 0b0101_1010);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0002);
                    assert_r8!(cpu, A, 0b1111_0101);
                    assert_flags!(cpu, f0, (S:1 Z:0 H:0 PV:1 N:0 C:0));
                });
            };
            ($fname:ident, A) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(XOR A);
                    cpu_eval!(cpu, A <- 0b0101_1010);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r8!(cpu, A, 0b0000_0000);
                    assert_flags!(cpu, f0, (S:0 Z:1 H:0 PV:1 N:0 C:0));
                });
            };
            ($fname:ident, $src:tt) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(XOR $src);
                    cpu_eval!(cpu, A <- 0b0101_1010);
                    cpu_eval!(cpu, $src <- 0b1010_1111);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r8!(cpu, A, 0b1111_0101);
                    assert_flags!(cpu, f0, (S:1 Z:0 H:0 PV:1 N:0 C:0));
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
        decl_test_case!(n, n);
    });

    decl_scenario!(exec_cp, {
        macro_rules! decl_test_case {
            ($fname:ident, n) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(CP 13);
                    cpu_eval!(cpu, A <- 42);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0002);
                    assert_r8!(cpu, A, 42);
                    assert_flags!(cpu, f0, (S:0 Z:0 H:1 PV:0 N:1 C:0));
                });
            };
            ($fname:ident, A) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(CP A);
                    cpu_eval!(cpu, A <- 42);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r8!(cpu, A, 42);
                    assert_flags!(cpu, f0, (S:0 Z:1 H:0 PV:0 N:1 C:0));
                });
            };
            ($fname:ident, $src:tt) => {
                decl_test!($fname, {
                    let mut cpu = cpu!(CP $src);
                    cpu_eval!(cpu, A <- 42);
                    cpu_eval!(cpu, $src <- 13);
                    let f0 = exec_step!(&mut cpu);
                    assert_pc!(cpu, 0x0001);
                    assert_r8!(cpu, A, 42);
                    assert_flags!(cpu, f0, (S:0 Z:0 H:1 PV:0 N:1 C:0));
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

    decl_test!(exec_jp_l16, {
        let mut cpu = cpu!(JP 0x4000);

        let f0 = exec_step!(&mut cpu);

        assert_pc!(cpu, 0x4000);
        assert_flags!(cpu, f0, unaffected);
    });

    decl_scenario!(exec_jp_cc_l16, {
        macro_rules! decl_test_case {
            ($cname:ident, $flag:ident, true) => {
                decl_test!($cname, {
                    let mut cpu = cpu!(JP $flag, 0x4000);
                    cpu_eval!(cpu, F +<- ($flag:1));

                    let f0 = exec_step!(&mut cpu);

                    assert_pc!(cpu, 0x4000);
                    assert_flags!(cpu, f0, unaffected);
                });
            };
            ($cname:ident, $flag:ident, false) => {
                decl_test!($cname, {
                    let mut cpu = cpu!(JP $flag, 0x2000);
                    cpu_eval!(cpu, F +<- ($flag:0));

                    let f0 = exec_step!(&mut cpu);

                    assert_pc!(cpu, 0x0003);
                    assert_flags!(cpu, f0, unaffected);
                });
            };
        }

        macro_rules! decl_test_suite {
            ($cname:ident, $flag:ident) => {
                decl_scenario!($cname, {
                    decl_test_case!(cond_unmet, $flag, false);
                    decl_test_case!(cond_met, $flag, true);
                });
            };
        }

        decl_test_suite!(nz, NZ);
        decl_test_suite!(nc, NC);
        decl_test_suite!(po, PO);
        decl_test_suite!(p, P);
        decl_test_suite!(z, Z);
        decl_test_suite!(c, C);
        decl_test_suite!(pe, PE);
        decl_test_suite!(m, M);
    });

    decl_scenario!(exec_jp_r16, {
        macro_rules! decl_test_case {
            ($cname:ident, $dst:tt) => {
                decl_test!($cname, {
                    let mut cpu = cpu!(JP ($dst));
                    cpu_eval!(cpu, $dst <- 0x4000);

                    let f0 = exec_step!(&mut cpu);

                    assert_pc!(cpu, 0x4000);
                    assert_flags!(cpu, f0, unaffected);
                });

            };
        }

        decl_test_case!(hl, HL);
    });

    /*************************/
    /* Call and return Group */
    /*************************/

    decl_scenario!(exec_call_cc_nn, {
        macro_rules! decl_test_case {
            ($cname:ident, $flag:ident, true) => {
                decl_test!($cname, {
                    let mut cpu = cpu!(CALL $flag, 0x4000);
                    cpu_eval!(cpu, F +<- ($flag:1));

                    let f0 = exec_step!(&mut cpu);

                    assert_pc!(cpu, 0x4000);
                    assert_r16!(cpu, SP, 0x7ffe);
                    assert_mem16!(cpu, 0x7ffe, 0x0003);
                    assert_flags!(cpu, f0, unaffected);
                });
            };
            ($cname:ident, $flag:ident, false) => {
                decl_test!($cname, {
                    let mut cpu = cpu!(CALL $flag, 0x4000);
                    let sp = cpu_eval!(cpu, SP);
                    cpu_eval!(cpu, F +<- ($flag:0));

                    let f0 = exec_step!(&mut cpu);

                    assert_pc!(cpu, 0x0003);
                    assert_r16!(cpu, SP, sp);
                    assert_flags!(cpu, f0, unaffected);
                });
            };
        }

        macro_rules! decl_test_suite {
            ($cname:ident, $flag:ident) => {
                decl_scenario!($cname, {
                    decl_test_case!(cond_unmet, $flag, false);
                    decl_test_case!(cond_met, $flag, true);
                });
            };
        }

        decl_test_suite!(nz, NZ);
        decl_test_suite!(nc, NC);
        decl_test_suite!(po, PO);
        decl_test_suite!(p, P);
        decl_test_suite!(z, Z);
        decl_test_suite!(c, C);
        decl_test_suite!(pe, PE);
        decl_test_suite!(m, M);
    });

    decl_test!(exec_call, {
        let mut cpu = cpu!(CALL 0x4000);

        let f0 = exec_step!(&mut cpu);

        assert_pc!(cpu, 0x4000);
        assert_r16!(cpu, SP, 0x7ffe);
        assert_mem16!(cpu, 0x7ffe, 0x0003);
        assert_flags!(cpu, f0, unaffected);
    });

    decl_scenario!(exec_ret_cc, {
        macro_rules! decl_test_case {
            ($cname:ident, $flag:ident, true) => {
                decl_test!($cname, {
                    let mut cpu = cpu!(RET $flag);
                    let sp = cpu_eval!(cpu, SP);
                    cpu_eval!(cpu, F +<- ($flag:1));
                    cpu_eval!(cpu, (**SP) <- 0x4000);

                    let f0 = exec_step!(&mut cpu);

                    assert_pc!(cpu, 0x4000);
                    assert_r16!(cpu, SP, sp + 2);
                    assert_flags!(cpu, f0, unaffected);
                });
            };
            ($cname:ident, $flag:ident, false) => {
                decl_test!($cname, {
                    let mut cpu = cpu!(RET $flag);
                    let sp = cpu_eval!(cpu, SP);
                    cpu_eval!(cpu, F +<- ($flag:0));
                    cpu_eval!(cpu, (**SP) <- 0x4000);

                    let f0 = exec_step!(&mut cpu);

                    assert_pc!(cpu, 0x0001);
                    assert_r16!(cpu, SP, sp);
                    assert_flags!(cpu, f0, unaffected);
                });
            };
        }

        macro_rules! decl_test_suite {
            ($cname:ident, $flag:ident) => {
                decl_scenario!($cname, {
                    decl_test_case!(cond_unmet, $flag, false);
                    decl_test_case!(cond_met, $flag, true);
                });
            };
        }

        decl_test_suite!(nz, NZ);
        decl_test_suite!(nc, NC);
        decl_test_suite!(po, PO);
        decl_test_suite!(p, P);
        decl_test_suite!(z, Z);
        decl_test_suite!(c, C);
        decl_test_suite!(pe, PE);
        decl_test_suite!(m, M);
    });

    decl_test!(exec_ret, {
        let mut cpu = cpu!(RET);
        cpu_eval!(cpu, (**SP) <- 0x4000);

        let f0 = exec_step!(&mut cpu);

        assert_pc!(cpu, 0x4000);
        assert_r16!(cpu, SP, 0x8002);
        assert_flags!(cpu, f0, unaffected);
    });

    decl_scenario!(exec_rst_n, {
        macro_rules! decl_test_case {
            ($cname:ident, $dst:tt) => {
                decl_test!($cname, {
                    let mut cpu = cpu!(RST $dst);

                    let f0 = exec_step!(&mut cpu);

                    assert_pc!(cpu, $dst);
                    assert_r16!(cpu, SP, 0x7ffe);
                    assert_mem16!(cpu, 0x7ffe, 0x0001);
                    assert_flags!(cpu, f0, unaffected);
                });
            };
        }

        decl_test_case!(_00h, 0x00);
        decl_test_case!(_08h, 0x08);
        decl_test_case!(_10h, 0x10);
        decl_test_case!(_18h, 0x18);
        decl_test_case!(_20h, 0x20);
        decl_test_case!(_28h, 0x28);
        decl_test_case!(_30h, 0x30);
        decl_test_case!(_38h, 0x38);
    });

    /**************************/
    /* Input and output Group */
    /**************************/

    decl_test!(exec_in_n_a, {
        let mut cpu = cpu!(IN A, (0x20));
        cpu_eval!(cpu, (!0x20) <- 0x12);

        let f0 = exec_step!(&mut cpu);

        assert_pc!(cpu, 0x0002);
        assert_r8!(cpu, A, 0x12);
        assert_flags!(cpu, f0, unaffected);
    });

    decl_test!(exec_out_n_a, {
        let mut cpu = cpu!(OUT (0x20), A);
        cpu_eval!(cpu, A <- 0x12);

        let f0 = exec_step!(&mut cpu);

        assert_pc!(cpu, 0x0002);
        assert_io!(cpu, 0x20, 0x12);
        assert_flags!(cpu, f0, unaffected);
    });
}

#[cfg(all(feature = "nightly", test))]
mod bench {
    use test;
    use test::Bencher;

    use crate::bus;
    use crate::cpu::z80;
    use crate::mem;

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
        let mut mem = Box::new(mem::MemoryBank::new());
        fill_instruction(&mut mem, inst, cycles);
        let io = Box::new(bus::Dead);
        let mut cpu = z80::CPU::new(z80::Options::default(), mem, io);
        b.iter(|| {
            cpu_eval!(cpu, PC <- 0);
            let mut total_cycles = 0;
            while total_cycles < cycles {
                total_cycles += test::black_box(exec_step(&mut cpu));
            }
        })
    }

    fn fill_instruction(mem: &mut mem::MemoryBank<u16, u8>, inst: &[u8], count: usize) {
        let mut addr = 0;
        for _ in 1..count {
            let mut src = inst;
            addr += mem.copy_to(addr, &mut src).unwrap() as u16;
        }
    }
}
