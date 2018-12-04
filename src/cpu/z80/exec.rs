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
        let pc = cpu_eval!(self, PC);
        let pos = ((pc as usize) + offset) as u16;
        self.mem().read_from(pos)
    }
}

// Returns the size of the given operand encoded in the instruction
macro_rules! op_size {
    // Something relative to the PC occupies 1 byte
    (*) => { 1 };
    // Something relative to the PC read as word occupies 2 byte
    (**) => { 2 };
    ((**)) => { 2 };
    ((**):u16) => { 2 };
    // Anything else occupies 0 bytes
    ($($_:tt)+) => { 0 };
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
        if flag!(C, flags) == 0 {
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
        if flag!(N, flags) == 0 {
            if flag!(H, flags) == 1 || a & 0x0f > 0x09 {
                a = $cpu.alu().add8(a, 0x06);
            }
            if flag!(C, flags) == 1 || a > 0x99 {
                a = $cpu.alu().add8(a, 0x60);
            }
        } else {
            if flag!(H, flags) == 1 || a & 0x0f > 0x09 {
                let (r, _) = $cpu.alu().sub8(a, 0x06);
                a = r;
            }
            if flag!(C, flags) == 1 || a > 0x99 {
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
            C:[flag!(C, flags) > 0 || prev_a > 0x99]
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
        let carry = flag!(C, cpu_eval!($cpu, F));
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
        let carry = flag!(C, cpu_eval!($cpu, F));
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



pub fn exec_step<CTX: Context>(ctx: &mut CTX) -> Cycles {
    let pc = cpu_eval!(ctx, PC);
    let opcode = ctx.read_from_pc(0);
    match opcode {
        0x00 => { cpu_exec!(ctx, NOP);              04 },
        0x01 => { cpu_exec!(ctx, LD BC, **);        10 },
        0x02 => { cpu_exec!(ctx, LD (BC), A);       07 },
        0x03 => { cpu_exec!(ctx, INC16 BC);         06 },
        0x04 => { cpu_exec!(ctx, INC8 B);           04 },
        0x05 => { cpu_exec!(ctx, DEC8 B);           04 },
        0x06 => { cpu_exec!(ctx, LD B, *);          07 },
        0x07 => { cpu_exec!(ctx, RLCA);             04 },
        0x08 => { cpu_exec!(ctx, EXAF);             04 },
        0x09 => { cpu_exec!(ctx, ADD16 HL, BC);     11 },
        0x0a => { cpu_exec!(ctx, LD A, (BC));       07 },
        0x0b => { cpu_exec!(ctx, DEC16 BC);         06 },
        0x0c => { cpu_exec!(ctx, INC8 C);           04 },
        0x0d => { cpu_exec!(ctx, DEC8 C);           04 },
        0x0e => { cpu_exec!(ctx, LD C, *);          07 },
        0x0f => { cpu_exec!(ctx, RRCA);             04 },
        0x10 => { if cpu_exec!(ctx, DJNZ) { 13 } else { 8 } },
        0x11 => { cpu_exec!(ctx, LD DE, **);        10 },
        0x12 => { cpu_exec!(ctx, LD (DE), A);       07 },
        0x13 => { cpu_exec!(ctx, INC16 DE);         06 },
        0x14 => { cpu_exec!(ctx, INC8 D);           04 },
        0x15 => { cpu_exec!(ctx, DEC8 D);           04 },
        0x16 => { cpu_exec!(ctx, LD D, *);          07 },
        0x17 => { cpu_exec!(ctx, RLA);              04 },
        0x18 => { cpu_exec!(ctx, JR *);             12 },
        0x19 => { cpu_exec!(ctx, ADD16 HL, DE);     11 },
        0x1a => { cpu_exec!(ctx, LD A, (DE));       07 },
        0x1b => { cpu_exec!(ctx, DEC16 DE);         06 },
        0x1c => { cpu_exec!(ctx, INC8 E);           04 },
        0x1d => { cpu_exec!(ctx, DEC8 E);           04 },
        0x1e => { cpu_exec!(ctx, LD E, *);          07 },
        0x1f => { cpu_exec!(ctx, RRA);              04 },
        0x20 => { cpu_exec!(ctx, JR NZ, *) },
        0x21 => { cpu_exec!(ctx, LD HL, **);        10 },
        0x22 => { cpu_exec!(ctx, LD (**):u16, HL);  16 },
        0x23 => { cpu_exec!(ctx, INC16 HL);         06 },
        0x24 => { cpu_exec!(ctx, INC8 H);           04 },
        0x25 => { cpu_exec!(ctx, DEC8 H);           04 },
        0x26 => { cpu_exec!(ctx, LD H, *);          07 },
        0x27 => { cpu_exec!(ctx, DAA);              04 },
        0x28 => { cpu_exec!(ctx, JR Z, *) },
        0x29 => { cpu_exec!(ctx, ADD16 HL, HL);     11 },
        0x2a => { cpu_exec!(ctx, LD HL, (**):u16);  16 },
        0x2b => { cpu_exec!(ctx, DEC16 HL);         06 },
        0x2c => { cpu_exec!(ctx, INC8 L);           04 },
        0x2d => { cpu_exec!(ctx, DEC8 L);           04 },
        0x2e => { cpu_exec!(ctx, LD L, *);          07 },
        0x2f => { cpu_exec!(ctx, CPL);              04 },
        0x30 => { cpu_exec!(ctx, JR NC, *) },
        0x31 => { cpu_exec!(ctx, LD SP, **);        10 },
        0x32 => { cpu_exec!(ctx, LD (**), A);       13 },
        0x33 => { cpu_exec!(ctx, INC16 SP);         06 },
        0x34 => { cpu_exec!(ctx, INC8 (HL));        11 },
        0x35 => { cpu_exec!(ctx, DEC8 (HL));        11 },
        0x36 => { cpu_exec!(ctx, LD (HL), *);       10 },
        0x37 => { cpu_exec!(ctx, SCF);              04 },
        0x38 => { cpu_exec!(ctx, JR C, *) },
        0x39 => { cpu_exec!(ctx, ADD16 HL, SP);     11 },
        0x3a => { cpu_exec!(ctx, LD A, (**));       13 },
        0x3b => { cpu_exec!(ctx, DEC16 SP);         06 },
        0x3c => { cpu_exec!(ctx, INC8 A);           04 },
        0x3d => { cpu_exec!(ctx, DEC8 A);           04 },
        0x3e => { cpu_exec!(ctx, LD A, *);          07 },
        0x3f => { cpu_exec!(ctx, CCF);              04 },
        0x40 => { cpu_exec!(ctx, LD B, B);          04 },
        0x41 => { cpu_exec!(ctx, LD B, C);          04 },
        0x42 => { cpu_exec!(ctx, LD B, D);          04 },
        0x43 => { cpu_exec!(ctx, LD B, E);          04 },
        0x44 => { cpu_exec!(ctx, LD B, H);          04 },
        0x45 => { cpu_exec!(ctx, LD B, L);          04 },
        0x46 => { cpu_exec!(ctx, LD B, (HL));       07 },
        0x47 => { cpu_exec!(ctx, LD B, A);          04 },
        0x48 => { cpu_exec!(ctx, LD C, B);          04 },
        0x49 => { cpu_exec!(ctx, LD C, C);          04 },
        0x4a => { cpu_exec!(ctx, LD C, D);          04 },
        0x4b => { cpu_exec!(ctx, LD C, E);          04 },
        0x4c => { cpu_exec!(ctx, LD C, H);          04 },
        0x4d => { cpu_exec!(ctx, LD C, L);          04 },
        0x4e => { cpu_exec!(ctx, LD C, (HL));       07 },
        0x4f => { cpu_exec!(ctx, LD C, A);          04 },
        0x50 => { cpu_exec!(ctx, LD D, B);          04 },
        0x51 => { cpu_exec!(ctx, LD D, C);          04 },
        0x52 => { cpu_exec!(ctx, LD D, D);          04 },
        0x53 => { cpu_exec!(ctx, LD D, E);          04 },
        0x54 => { cpu_exec!(ctx, LD D, H);          04 },
        0x55 => { cpu_exec!(ctx, LD D, L);          04 },
        0x56 => { cpu_exec!(ctx, LD D, (HL));       07 },
        0x57 => { cpu_exec!(ctx, LD D, A);          04 },
        0x58 => { cpu_exec!(ctx, LD E, B);          04 },
        0x59 => { cpu_exec!(ctx, LD E, C);          04 },
        0x5a => { cpu_exec!(ctx, LD E, D);          04 },
        0x5b => { cpu_exec!(ctx, LD E, E);          04 },
        0x5c => { cpu_exec!(ctx, LD E, H);          04 },
        0x5d => { cpu_exec!(ctx, LD E, L);          04 },
        0x5e => { cpu_exec!(ctx, LD E, (HL));       07 },
        0x5f => { cpu_exec!(ctx, LD E, A);          04 },
        0x60 => { cpu_exec!(ctx, LD H, B);          04 },
        0x61 => { cpu_exec!(ctx, LD H, C);          04 },
        0x62 => { cpu_exec!(ctx, LD H, D);          04 },
        0x63 => { cpu_exec!(ctx, LD H, E);          04 },
        0x64 => { cpu_exec!(ctx, LD H, H);          04 },
        0x65 => { cpu_exec!(ctx, LD H, L);          04 },
        0x66 => { cpu_exec!(ctx, LD H, (HL));       07 },
        0x67 => { cpu_exec!(ctx, LD H, A);          04 },
        0x68 => { cpu_exec!(ctx, LD L, B);          04 },
        0x69 => { cpu_exec!(ctx, LD L, C);          04 },
        0x6a => { cpu_exec!(ctx, LD L, D);          04 },
        0x6b => { cpu_exec!(ctx, LD L, E);          04 },
        0x6c => { cpu_exec!(ctx, LD L, H);          04 },
        0x6d => { cpu_exec!(ctx, LD L, L);          04 },
        0x6e => { cpu_exec!(ctx, LD L, (HL));       07 },
        0x6f => { cpu_exec!(ctx, LD L, A);          04 },
        0x70 => { cpu_exec!(ctx, LD (HL), B);       07 },
        0x71 => { cpu_exec!(ctx, LD (HL), C);       07 },
        0x72 => { cpu_exec!(ctx, LD (HL), D);       07 },
        0x73 => { cpu_exec!(ctx, LD (HL), E);       07 },
        0x74 => { cpu_exec!(ctx, LD (HL), H);       07 },
        0x75 => { cpu_exec!(ctx, LD (HL), L);       07 },
        0x76 => { cpu_exec!(ctx, HALT);             04 },
        0x77 => { cpu_exec!(ctx, LD (HL), A);       07 },
        0x78 => { cpu_exec!(ctx, LD A, B);          04 },
        0x79 => { cpu_exec!(ctx, LD A, C);          04 },
        0x7a => { cpu_exec!(ctx, LD A, D);          04 },
        0x7b => { cpu_exec!(ctx, LD A, E);          04 },
        0x7c => { cpu_exec!(ctx, LD A, H);          04 },
        0x7d => { cpu_exec!(ctx, LD A, L);          04 },
        0x7e => { cpu_exec!(ctx, LD A, (HL));       07 },
        0x7f => { cpu_exec!(ctx, LD A, A);          04 },
        0x80 => { cpu_exec!(ctx, ADD8 A, B);        04 },
        0x81 => { cpu_exec!(ctx, ADD8 A, C);        04 },
        0x82 => { cpu_exec!(ctx, ADD8 A, D);        04 },
        0x83 => { cpu_exec!(ctx, ADD8 A, E);        04 },
        0x84 => { cpu_exec!(ctx, ADD8 A, H);        04 },
        0x85 => { cpu_exec!(ctx, ADD8 A, L);        04 },
        0x86 => { cpu_exec!(ctx, ADD8 A, (HL));     07 },
        0x87 => { cpu_exec!(ctx, ADD8 A, A);        04 },
        0x88 => { cpu_exec!(ctx, ADC8 A, B);        04 },
        0x89 => { cpu_exec!(ctx, ADC8 A, C);        04 },
        0x8a => { cpu_exec!(ctx, ADC8 A, D);        04 },
        0x8b => { cpu_exec!(ctx, ADC8 A, E);        04 },
        0x8c => { cpu_exec!(ctx, ADC8 A, H);        04 },
        0x8d => { cpu_exec!(ctx, ADC8 A, L);        04 },
        0x8e => { cpu_exec!(ctx, ADC8 A, (HL));     07 },
        0x8f => { cpu_exec!(ctx, ADC8 A, A);        04 },

        0xc3 => { cpu_exec!(ctx, JP **);            10 },
        _ => unimplemented!("cannot execute illegal instruction with opcode 0x{:x}", opcode),
    }
}

#[cfg(test)]
mod test {
    use std::fmt;
    use std::io::Write;

    use rand;

    use cpu::z80;

    use super::*;

    macro_rules! cpu {
        () => {
            {
                let prev_flags = u8::sample() & 0b11010111; // Do not set F5 and F3
                let mem = z80::MemoryBank::new();
                let mut cpu = z80::CPU::new(z80::Options::default(), mem);
                cpu_eval!(cpu, F <- prev_flags);
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

    macro_rules! decl_test {
        ($fname:ident, $body:block) => {
            #[test]
            fn $fname() {
                $body
            }
        };
    }

    // Produces a setter function for the given destination
    macro_rules! setter {
        ($dest:tt) => (|val, cpu: &mut CPU| cpu_eval!(cpu, $dest <- val));
    }
    
    // Produces a setup function to prepare the 8-bits destination
    macro_rules! setup_dst {
        (($a:ident)) => (|_, cpu: &mut CPU| { setter!($a)(0x1234, cpu); });
        ($a:tt) => (|_, _: &mut CPU| {});
    }

    // Produces a setup function to prepare the 8-bits source
    macro_rules! setup_src8 {
        (inst => $a:expr) => (|val: u8, cpu: &mut CPU| cpu.mem_mut().write(&$a(val)).unwrap());
        (($a:ident)) => (|val: u8, cpu: &mut CPU| { 
            setter!((0x1234))(val, cpu);
            setter!($a)(0x1234, cpu);
        });
        (($a:expr)) => (|val: u8, cpu: &mut CPU| { 
            setter!(($a))(val, cpu);
        });
        ($a:tt) => (setter!($a));
    }

    // Produces a setup function to prepare the 16-bits source
    macro_rules! setup_src16 {
        (($a:ident)) => (|val: u16, cpu: &mut CPU| { 
            setter!((0x1234): u16)(val, cpu);
            setter!($a)(0x1234, cpu);
        });
        (($a:expr)) => (|val: u16, cpu: &mut CPU| { 
            setter!(($a): u16)(val, cpu);
        });
        ($a:tt) => (setter!($a));
    }

    // Produces a setup function to prepare the instruction
    macro_rules! setup_inst {
        ($a:expr) => (|val: u16, cpu: &mut CPU| cpu.mem_mut().write(&$a(val)).unwrap());
    }

    // Produces a setup function that combines different setup functions for unary operations
    macro_rules! setup_unary {
        ($( $a:expr ),+) => (|val, cpu: &mut CPU| {
            $( $a(val, cpu) );+
        })
    }

    // Produces a setup function that combines different setup functions for binary operations
    macro_rules! setup_binary {
        ($a:expr, $b:expr) => (|a, b, cpu: &mut CPU| {
            $a(a, cpu);
            $b(b, cpu);
        })
    }

    // Produces a setup function to prepare the CPU flags
    macro_rules! setup_flags {
        ($cpu:expr, $( $flags:tt )*) => ({
            let mut flags = cpu_eval!($cpu, F);
            flags = flags_apply!(flags, $( $flags )*);
            cpu_eval!($cpu, F <- flags);
        })
    }

    // Produces a getter function from the given source
    macro_rules! getter {
        ($( $src:tt )+) => (|cpu: &CPU| cpu_eval!(cpu, $( $src )+));
    }

    // Set a random value as source operand and save it to the given source setter
    macro_rules! random_src {
        ($type:ty, $cpu:expr, $srcset:expr) => ({
            let input = <$type>::sample();
            $srcset(input, $cpu);
            input
        })
    }

    macro_rules! expected_flags {
        ($($flags:tt)*) => (|f| flags_apply!(f, $( $flags )* ))
    }

    macro_rules! assert_behaves_like_load {
        ($type:ty, $pcinc:expr, $cpu:expr, $srcset:expr, $dstget:expr) => {
            let input = random_src!($type, $cpu, $srcset);
            let f0 = exec_step!($cpu);
            assert_program_counter!($cpu, $pcinc);
            assert_dest!($type, $cpu, $dstget, input);
            assert_flags!(unaffected, $cpu, f0);
        };
    }    

    /********************/
    /* 8-Bit Load Group */
    /********************/

    macro_rules! assert_behaves_like_load8 {
        ($pcinc:expr, $cpu:expr, $srcset:expr, $dstget:expr) => {
            assert_behaves_like_load!(u8, $pcinc, $cpu, $srcset, $dstget);
        };
    }    

    macro_rules! test_exec_ld8 {
        ($fname:ident, $pcinc:expr, $dstname:tt, *) => {
            decl_test!($fname, {
                let mut cpu = cpu!();
                assert_behaves_like_load8!($pcinc, &mut cpu,
                    setup_unary!(
                        setup_dst!($dstname), 
                        setup_src8!(inst => |val| inst!(LD $dstname, val))
                    ),
                    getter!($dstname)
                );
            });
        };
        ($fname:ident, $pcinc:expr, $dstname:tt, $srcname:tt) => {
            decl_test!($fname, {
                let mut cpu = cpu!(LD $dstname, $srcname);
                assert_behaves_like_load8!($pcinc, &mut cpu,
                    setup_unary!(setup_dst!($dstname), setup_src8!($srcname)),
                    getter!($dstname)
                );
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

    test_exec_ld8!(test_exec_ld8_indbc_a, 1, (BC), A);
    test_exec_ld8!(test_exec_ld8_indde_a, 1, (DE), A);
    test_exec_ld8!(test_exec_ld8_indhl_a, 1, (HL), A);
    test_exec_ld8!(test_exec_ld8_indhl_b, 1, (HL), B);
    test_exec_ld8!(test_exec_ld8_indhl_c, 1, (HL), C);
    test_exec_ld8!(test_exec_ld8_indhl_d, 1, (HL), D);
    test_exec_ld8!(test_exec_ld8_indhl_e, 1, (HL), E);
    test_exec_ld8!(test_exec_ld8_indhl_h, 1, (HL), H);
    test_exec_ld8!(test_exec_ld8_indhl_l, 1, (HL), L);

    test_exec_ld8!(test_exec_ld8_a_indbc, 1, A, (BC));
    test_exec_ld8!(test_exec_ld8_a_indde, 1, A, (DE));
    test_exec_ld8!(test_exec_ld8_a_indhl, 1, A, (HL));
    test_exec_ld8!(test_exec_ld8_b_indhl, 1, B, (HL));
    test_exec_ld8!(test_exec_ld8_c_indhl, 1, C, (HL));
    test_exec_ld8!(test_exec_ld8_d_indhl, 1, D, (HL));
    test_exec_ld8!(test_exec_ld8_e_indhl, 1, E, (HL));
    test_exec_ld8!(test_exec_ld8_h_indhl, 1, H, (HL));
    test_exec_ld8!(test_exec_ld8_l_indhl, 1, L, (HL));

    test_exec_ld8!(test_exec_ld8_indl16_a, 3, (0x1234), A);

    test_exec_ld8!(test_exec_ld8_a_l8, 2, A, *);
    test_exec_ld8!(test_exec_ld8_b_l8, 2, B, *);
    test_exec_ld8!(test_exec_ld8_c_l8, 2, C, *);
    test_exec_ld8!(test_exec_ld8_d_l8, 2, D, *);
    test_exec_ld8!(test_exec_ld8_e_l8, 2, E, *);
    test_exec_ld8!(test_exec_ld8_h_l8, 2, H, *);
    test_exec_ld8!(test_exec_ld8_l_l8, 2, L, *);

    test_exec_ld8!(test_exec_ld8_indhl_l8, 2, (HL), *);

    test_exec_ld8!(test_ld_a_indl16, 3, A, (0x1234));
        

    /*********************/
    /* 16-Bit Load Group */
    /*********************/

    macro_rules! assert_behaves_like_load16 {
        ($pcinc:expr, $cpu:expr, $srcset:expr, $dstget:expr) => {
            assert_behaves_like_load!(u16, $pcinc, $cpu, $srcset, $dstget);
        };
    }    

    macro_rules! test_exec_ld16 {
        ($fname:ident, $pcinc:expr, $dstname:tt, **) => {
            decl_test!($fname, {
                let mut cpu = cpu!();
                assert_behaves_like_load16!($pcinc, &mut cpu,
                    setup_unary!(
                        setup_dst!($dstname), 
                        setup_inst!(|val| inst!(LD $dstname, val))
                    ),
                    getter!($dstname)
                );
            });
        };
        ($fname:ident, $pcinc:expr, (**), $srcname:tt) => {
            decl_test!($fname, {
                let mut cpu = cpu!(LD (0x1234), $srcname);
                assert_behaves_like_load16!($pcinc, &mut cpu,
                    setup_src16!($srcname),
                    getter!((0x1234): u16)
                );
            });
        };
        ($fname:ident, $pcinc:expr, $dstname:tt, $srcname:tt) => {
            decl_test!($fname, {
                let mut cpu = cpu!(&inst!(LD $dstname, $srcname));
                assert_behaves_like_load16!($pcinc, &mut cpu,
                    setup_unary!(setup_dst!($dstname), setup_src16!($srcname)),
                    getter!($dstname)
                );
            });
        };
    }

    test_exec_ld16!(test_exec_ld16_bc_l16, 3, BC, **);
    test_exec_ld16!(test_exec_ld16_de_l16, 3, DE, **);
    test_exec_ld16!(test_exec_ld16_hl_l16, 3, HL, **);
    test_exec_ld16!(test_exec_ld16_sp_l16, 3, SP, **);

    test_exec_ld16!(test_exec_ld16_indl16_hl, 3, (**), HL);

    test_exec_ld16!(test_exec_ld16_hl_indl16, 3, HL, **);

    /**********************************************/
    /* Exchange, Block Transfer, and Search Group */
    /**********************************************/

    decl_test!(test_exec_exaf, {
        let mut cpu = cpu!(EX AF, AF_);
        let af = random_src!(u16, &mut cpu, setter!(AF));
        let af_ = random_src!(u16, &mut cpu, setter!(AF_));

        exec_step!(&mut cpu);
        assert_program_counter!(cpu, 0x0001);
        assert_r16!(&mut cpu, AF, af_);
        assert_r16!(&mut cpu, AF_, af);
    });

    /**************************/
    /* 8-Bit Arithmetic group */
    /**************************/

    macro_rules! assert_behaves_like_add8 {
        ($pcinc:expr, $cpu:expr, $srcset:expr, $dstget:expr) => {
            struct Case {
                name: &'static str,
                input: u8,
                expected: u8,
                expected_flags: fn(u8) -> u8,
            }
            table_test!(&[
                Case {
                    name: "Regular case",
                    input: 0x21, 
                    expected: 0x42, 
                    expected_flags: expected_flags!(S:0 Z:0 H:0 PV:0 N:0 C:0),
                },
                Case {
                    name: "Overflow + signed",
                    input: 0x51,
                    expected: 0xa2,
                    expected_flags: expected_flags!(S:1 Z:0 H:0 PV:1 N:0 C:0),
                },
                Case {
                    name: "Half carry",
                    input: 0x29,
                    expected: 0x52,
                    expected_flags: expected_flags!(S:0 Z:0 H:1 PV:0 N:0 C:0),
                },
                Case {
                    name: "Zero",
                    input: 0,
                    expected: 0,
                    expected_flags: expected_flags!(S:0 Z:1 H:0 PV:0 N:0 C:0),
                },
                Case {
                    name: "Carry",
                    input: 0x90,
                    expected: 0x20,
                    expected_flags: expected_flags!(S:0 Z:0 H:0 PV:1 N:0 C:1),
                },
            ], |case: &Case| {
                $srcset(case.input, case.input, $cpu);
                let f0 = exec_step!($cpu);
                assert_program_counter!($cpu, 0x0001);
                assert_dest!(u8, $cpu, $dstget, case.expected);
                assert_flags!($cpu, case.expected_flags, f0);
            });
        };
    }

    macro_rules! assert_behaves_like_adc8 {
        ($pcinc:expr, $cpu:expr, $srcset:expr, $dstget:expr) => {
            struct Case {
                name: &'static str,
                input: u8,
                carry: u8,
                expected: u8,
                expected_flags: fn(u8) -> u8,
            }
            table_test!(&[
                Case {
                    name: "Regular case, no prev carry",
                    input: 0x21,
                    carry: 0,
                    expected: 0x42,
                    expected_flags: expected_flags!(S:0 Z:0 H:0 PV:0 N:0 C:0),
                },
                Case {
                    name: "Regular case, prev carry",
                    input: 0x21,
                    carry: 1,
                    expected: 0x43,
                    expected_flags: expected_flags!(S:0 Z:0 H:0 PV:0 N:0 C:0),
                },
                Case {
                    name: "Overflow + signed",
                    input: 0x51,
                    carry: 0,
                    expected: 0xa2,
                    expected_flags: expected_flags!(S:1 Z:0 H:0 PV:1 N:0 C:0),
                },
                Case {
                    name: "Half carry",
                    input: 0x29,
                    carry: 0,
                    expected: 0x52,
                    expected_flags: expected_flags!(S:0 Z:0 H:1 PV:0 N:0 C:0),
                },
                Case {
                    name: "Zero",
                    input: 0,
                    carry: 0,
                    expected: 0,
                    expected_flags: expected_flags!(S:0 Z:1 H:0 PV:0 N:0 C:0),
                },
                Case {
                    name: "Carry",
                    input: 0x90,
                    carry: 0,
                    expected: 0x20,
                    expected_flags: expected_flags!(S:0 Z:0 H:0 PV:1 N:0 C:1),
                },
            ], |case: &Case| {
                $srcset(case.input, case.input, $cpu);
                setup_flags!($cpu, C:[case.carry == 1]);
                let f0 = exec_step!($cpu);
                assert_program_counter!($cpu, 0x0001);
                assert_dest!(u8, $cpu, $dstget, case.expected);
                assert_flags!($cpu, case.expected_flags, f0);
            });
        };
    }

    macro_rules! assert_behaves_like_inc8 {
        ($pcinc:expr, $cpu:expr, $srcset:expr, $dstget:expr) => {
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
                    expected_flags: expected_flags!(S:0 Z:0 H:0 PV:0 N:0),
                },
                Case {
                    name: "Half-carry",
                    input: 0x0f,
                    expected: 0x10,
                    expected_flags: expected_flags!(S:0 Z:0 H:1 PV:0 N:0),
                },
                Case {
                    name: "Overflow",
                    input: 0x7f,
                    expected: 0x80,
                    expected_flags: expected_flags!(S:1 Z:0 H:1 PV:1 N:0),
                },
                Case {
                    name: "Carry",
                    input: 0xff,
                    expected: 0x00,
                    expected_flags: expected_flags!(S:0 Z:1 H:1 PV:0 N:0),
                },
            ], |case: &Case| {
                $srcset(case.input, $cpu);
                let f0 = exec_step!($cpu);
                assert_program_counter!($cpu, 0x0001);
                assert_dest!(u8, $cpu, $dstget, case.expected);
                assert_flags!($cpu, case.expected_flags, f0);
            });
        };
    }

    macro_rules! assert_behaves_like_dec8 {
        ($pcinc:expr, $cpu:expr, $srcset:expr, $dstget:expr) => {
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
                    expected_flags: expected_flags!(S:0 Z:0 H:1 PV:0 N:1),
                },
                Case {
                    name: "Overflow",
                    input: 0x80,
                    expected: 0x7f,
                    expected_flags: expected_flags!(S:0 Z:0 H:1 PV:1 N:1),
                },
                Case {
                    name: "Zero",
                    input: 0x01,
                    expected: 0x00,
                    expected_flags: expected_flags!(S:0 Z:1 H:0 PV:0 N:1),
                },
                Case {
                    name: "No carry",
                    input: 0x00,
                    expected: 0xff,
                    expected_flags: expected_flags!(S:1 Z:0 H:1 PV:0 N:1),
                },
            ], |case: &Case| {
                $srcset(case.input, $cpu);
                let f0 = exec_step!($cpu);
                assert_program_counter!($cpu, 0x0001);
                assert_dest!(u8, $cpu, $dstget, case.expected);
                assert_flags!($cpu, case.expected_flags, f0);
            });
        };
    }

    macro_rules! test_exec_add8 {
        ($fname:ident, $pcinc:expr, $dstname:tt, $srcname:tt) => {
            decl_test!($fname, {
                let mut cpu = cpu!(ADD $dstname, $srcname);
                assert_behaves_like_add8!($pcinc, &mut cpu,
                    setup_binary!(
                        setup_src8!($srcname), 
                        setup_src8!($dstname)
                    ),
                    getter!($dstname)
                );
            });
        };
    }

    macro_rules! test_exec_adc8 {
        ($fname:ident, $pcinc:expr, $dstname:tt, $srcname:tt) => {
            decl_test!($fname, {
                let mut cpu = cpu!(ADC $dstname, $srcname);
                assert_behaves_like_adc8!($pcinc, &mut cpu,
                    setup_binary!(
                        setup_src8!($srcname), 
                        setup_src8!($dstname)
                    ),
                    getter!($dstname)
                );
            });
        };
    }

    macro_rules! test_exec_inc8 {
        ($fname:ident, $pcinc:expr, $dstname:tt) => {
            decl_test!($fname, {
                let mut cpu = cpu!(INC $dstname);
                assert_behaves_like_inc8!($pcinc, &mut cpu,
                    setup_src8!($dstname),
                    getter!($dstname)
                );
            });
        };
    }

    macro_rules! test_exec_dec8 {
        ($fname:ident, $pcinc:expr, $dstname:tt) => {
            decl_test!($fname, {
                let mut cpu = cpu!(DEC $dstname);
                assert_behaves_like_dec8!($pcinc, &mut cpu,
                    setup_src8!($dstname),
                    getter!($dstname)
                );
            });
        };
    }

    test_exec_add8!(test_exec_add_a_a, 1, A, A);
    test_exec_add8!(test_exec_add_a_b, 1, A, B);
    test_exec_add8!(test_exec_add_a_c, 1, A, C);
    test_exec_add8!(test_exec_add_a_d, 1, A, D);
    test_exec_add8!(test_exec_add_a_e, 1, A, E);
    test_exec_add8!(test_exec_add_a_h, 1, A, H);
    test_exec_add8!(test_exec_add_a_l, 1, A, L);

    test_exec_add8!(test_exec_add_a_indhl, 1, A, (HL));

    test_exec_adc8!(test_exec_adc_a_a, 1, A, A);
    test_exec_adc8!(test_exec_adc_a_b, 1, A, B);
    test_exec_adc8!(test_exec_adc_a_c, 1, A, C);
    test_exec_adc8!(test_exec_adc_a_d, 1, A, D);
    test_exec_adc8!(test_exec_adc_a_e, 1, A, E);
    test_exec_adc8!(test_exec_adc_a_h, 1, A, H);
    test_exec_adc8!(test_exec_adc_a_l, 1, A, L);

    test_exec_adc8!(test_exec_adc_a_indhl, 3, A, (HL));

    test_exec_inc8!(test_exec_inc_a, 1, A);
    test_exec_inc8!(test_exec_inc_c, 1, C);
    test_exec_inc8!(test_exec_inc_d, 1, D);
    test_exec_inc8!(test_exec_inc_e, 1, E);
    test_exec_inc8!(test_exec_inc_h, 1, H);
    test_exec_inc8!(test_exec_inc_l, 1, L);

    test_exec_inc8!(test_exec_inc_indhl, 1, (HL));

    test_exec_dec8!(test_exec_dec_a, 1, A);
    test_exec_dec8!(test_exec_dec_b, 1, B);
    test_exec_dec8!(test_exec_dec_c, 1, C);
    test_exec_dec8!(test_exec_dec_d, 1, D);
    test_exec_dec8!(test_exec_dec_e, 1, E);
    test_exec_dec8!(test_exec_dec_h, 1, H);
    test_exec_dec8!(test_exec_dec_l, 1, L);

    test_exec_dec8!(test_exec_dec_indhl, 1, (HL));    

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

    #[test]
    fn test_exec_scf() {
        let mut test = ExecTest::for_inst(&inst!(SCF));
        test.exec_step();
        
        test.assert_hflag_if("SCF", false);
        test.assert_nflag_if("SCF", false);
        test.assert_cflag_if("SCF", true);
    }

    #[test]
    fn test_exec_ccf() {
        let mut test = ExecTest::for_inst(&inst!(CCF));
        // Case: flag C is reset
        test.cpu.regs_mut().set_flags(0x00);
        test.exec_step();
        
        test.assert_hflag_if("CCF", false);
        test.assert_nflag_if("CCF", false);
        test.assert_cflag_if("CCF", true);

        // Case: flag C is set
        test.cpu.regs_mut().set_flags(0x01);
        test.exec_step();
        
        test.assert_hflag_if("CCF", true);
        test.assert_nflag_if("CCF", false);
        test.assert_cflag_if("CCF", false);
    }

    #[test]
    fn test_exec_halt() {
        let mut test = ExecTest::for_inst(&inst!(HALT));
        test.exec_step();
        assert_result!(HEX16, "program counter", 0x0000, test.cpu.regs().pc());
        test.assert_all_flags_unaffected("halt");
    }

    /***************************/
    /* 16-Bit Arithmetic group */
    /***************************/

    macro_rules! test_inc_reg16 {
        ($fname:ident, $regname:ident, $regget:ident, $regset:ident) => {
            decl_test!($fname, {
                let mut test = ExecTest::for_inst(&inst!(INC $regname));
                test.assert_behaves_like_inc16(
                    |v, cpu| cpu.regs_mut().$regset(v),
                    |cpu| cpu.regs().$regget(),
                );
            });
        }
    }

    test_inc_reg16!(test_exec_inc_bc, BC, bc, set_bc);
    test_inc_reg16!(test_exec_inc_de, DE, de, set_de);
    test_inc_reg16!(test_exec_inc_hl, HL, hl, set_hl);
    test_inc_reg16!(test_exec_inc_sp, SP, sp, set_sp);

    macro_rules! test_dec_reg16 {
        ($fname:ident, $regname:ident, $regget:ident, $regset:ident) => {
            decl_test!($fname, {
                let mut test = ExecTest::for_inst(&inst!(DEC $regname));
                test.assert_behaves_like_dec16(
                    |v, cpu| cpu.regs_mut().$regset(v),
                    |cpu| cpu.regs().$regget(),
                );
            });
        }
    }

    test_dec_reg16!(test_exec_dec_bc, BC, bc, set_bc);
    test_dec_reg16!(test_exec_dec_de, DE, de, set_de);
    test_dec_reg16!(test_exec_dec_hl, HL, hl, set_hl);
    test_dec_reg16!(test_exec_dec_sp, SP, sp, set_sp);

    macro_rules! test_add_reg16_reg16 {
        ($fname:ident, $dstname:ident, $srcname:ident,
         $dstget:ident, $dstset:ident, $srcset:ident) => {
            decl_test!($fname, {
                let mut test = ExecTest::for_inst(&inst!(ADD $dstname, $srcname));
                test.asset_behaves_like_add16(
                    |a, b, cpu| {
                        cpu.regs_mut().$dstset(a);
                        cpu.regs_mut().$srcset(b);
                    },
                    |cpu| cpu.regs().$dstget(),
                );
            });
        }
    }

    test_add_reg16_reg16!(test_exec_add_hl_bc, HL, BC, hl, set_hl, set_bc);
    test_add_reg16_reg16!(test_exec_add_hl_de, HL, DE, hl, set_hl, set_de);
    test_add_reg16_reg16!(test_exec_add_hl_sp, HL, SP, hl, set_hl, set_sp);

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
            decl_test!($fname, {
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
            });
        }
    }

    test_jr_cond_l8!(test_exec_jr_c_l8, C, flag_c, |f| f | 0b00000001, |f| f & 0b11111110);
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

        fn assert_behaves_like_add8<S, G>(&mut self, set: S, get: G)
        where S: Fn(u8, u8, &mut CPU), G: Fn(&CPU) -> u8 {
            struct Case {
                name: &'static str,
                a: u8,
                b: u8,
                expected: u8,
                expected_flags: fn(u8) -> u8,
            }
            table_test!(&[
                Case {
                    name: "Regular case",
                    a: 0x21,
                    b: 0x21,
                    expected: 0x42,
                    expected_flags: |f| flags_apply!(f, S:0 Z:0 H:0 PV:0 N:0 C:0),
                },
                Case {
                    name: "Overflow + signed",
                    a: 0x51,
                    b: 0x51,
                    expected: 0xa2,
                    expected_flags: |f| flags_apply!(f, S:1 Z:0 H:0 PV:1 N:0 C:0),
                },
                Case {
                    name: "Half carry",
                    a: 0x29,
                    b: 0x29,
                    expected: 0x52,
                    expected_flags: |f| flags_apply!(f, S:0 Z:0 H:1 PV:0 N:0 C:0),
                },
                Case {
                    name: "Zero",
                    a: 0,
                    b: 0,
                    expected: 0,
                    expected_flags: |f| flags_apply!(f, S:0 Z:1 H:0 PV:0 N:0 C:0),
                },
                Case {
                    name: "Carry",
                    a: 0x90,
                    b: 0x90,
                    expected: 0x20,
                    expected_flags: |f| flags_apply!(f, S:0 Z:0 H:0 PV:1 N:0 C:1),
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

        fn assert_behaves_like_adc8<S, G>(&mut self, set: S, get: G)
        where S: Fn(u8, u8, &mut CPU), G: Fn(&CPU) -> u8 {
            struct Case {
                name: &'static str,
                a: u8,
                b: u8,
                c: u8,
                expected: u8,
                expected_flags: fn(u8) -> u8,
            }
            table_test!(&[
                Case {
                    name: "Regular case, no prev carry",
                    a: 0x21,
                    b: 0x21,
                    c: 0,
                    expected: 0x42,
                    expected_flags: |f| flags_apply!(f, S:0 Z:0 H:0 PV:0 N:0 C:0),
                },
                Case {
                    name: "Regular case, prev carry",
                    a: 0x21,
                    b: 0x21,
                    c: 1,
                    expected: 0x43,
                    expected_flags: |f| flags_apply!(f, S:0 Z:0 H:0 PV:0 N:0 C:0),
                },
                Case {
                    name: "Overflow + signed",
                    a: 0x51,
                    b: 0x51,
                    c: 0,
                    expected: 0xa2,
                    expected_flags: |f| flags_apply!(f, S:1 Z:0 H:0 PV:1 N:0 C:0),
                },
                Case {
                    name: "Half carry",
                    a: 0x29,
                    b: 0x29,
                    c: 0,
                    expected: 0x52,
                    expected_flags: |f| flags_apply!(f, S:0 Z:0 H:1 PV:0 N:0 C:0),
                },
                Case {
                    name: "Zero",
                    a: 0,
                    b: 0,
                    c: 0,
                    expected: 0,
                    expected_flags: |f| flags_apply!(f, S:0 Z:1 H:0 PV:0 N:0 C:0),
                },
                Case {
                    name: "Carry",
                    a: 0x90,
                    b: 0x90,
                    c: 0,
                    expected: 0x20,
                    expected_flags: |f| flags_apply!(f, S:0 Z:0 H:0 PV:1 N:0 C:1),
                },
            ], |case: &Case| {
                set(case.a, case.b, &mut self.cpu);
                let mut flags = self.cpu.regs().flags();
                flags = flags_apply!(flags, C:[case.c == 1]);
                self.cpu.regs_mut().set_flags(flags);
                self.exec_step();

                assert_result!(HEX16, "program counter", 0x0001, self.cpu.regs().pc());
                let actual = get(&self.cpu);
                let expected_flags = (case.expected_flags)(self.cpu.regs().flags());
                let actual_flags = self.cpu.regs().flags();
                assert_result!(HEX16, "dest", case.expected, actual);
                assert_result!(BIN8, "flags", expected_flags, actual_flags);
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
