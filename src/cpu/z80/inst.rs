use std::io;

use byteorder::{ReadBytesExt, LittleEndian};

use bus::{Address, Memory, MemoryItem};
use cpu::z80::data::{Data, Word, Byte};
use cpu::z80::regs::{Reg8, Reg16, Register, Registers};

// Context trait defines a context where instructions are executed
pub trait Context {
    type Mem: Memory;
    fn regs(&self) -> &Registers;
    fn regs_mut(&mut self) -> &mut Registers;
    fn mem(&self) -> &Self::Mem;
    fn mem_mut(&mut self) -> &mut Self::Mem;
}

pub trait OpRead<T> {
    fn read<C: Context>(&self, c: &C) -> T;
}

pub trait OpWrite<T> {
    fn write<C: Context>(&self, c: &mut C, val: T);
}

// Src defines a source operand of a instruction
#[derive(Debug, Eq, PartialEq)]
pub enum Src<D: Data> {
    Liter(D::Value),
    Reg(D::Reg),
    IndReg(Reg16),
}

type Src8 = Src<Byte>;
type Src16 = Src<Word>;

impl<D: Data> Src<D> {
    fn read<C: Context>(&self, c: &C) -> D::Value {
        match self {
            Src::Liter(v) => *v,
            Src::Reg(r) => r.read(c.regs()),
            Src::IndReg(r) => {
                let addr = Address::from(r.read(c.regs()) as u16);
                D::Value::mem_read(c.mem(), addr)
            },
        }
    }
}

// Dest defines a destination operand of a instruction
#[derive(Debug, Eq, PartialEq)]
pub enum Dest<D: Data> {
    Reg(D::Reg),
    IndReg(Reg16),
}

impl<D: Data> Dest<D> {
    fn read<C: Context>(&self, c: &C) -> D::Value {
        match self {
            Dest::Reg(r) => r.read(c.regs()),
            Dest::IndReg(r) => {
                let addr = Address::from(r.read(c.regs()) as u16);
                D::Value::mem_read(c.mem(), addr)
            },
        }
    }

    fn write<C: Context>(&self, c: &mut C, val: D::Value) {
        match self {
            Dest::Reg(r) => r.write(c.regs_mut(), val),
            Dest::IndReg(r) => {
                let addr = Address::from(r.read(c.regs()) as u16);
                D::Value::mem_write(c.mem_mut(), addr, val)
            },
        }
    }
}

type Dest8 = Dest<Byte>;
type Dest16 = Dest<Word>;

#[derive(Debug, Eq, PartialEq)]
pub enum Mnemo { ADD, DEC, EX, INC, LD, NOP, RLCA, RRCA }

#[derive(Debug, Eq, PartialEq)]
pub enum Operands {
    Nulary,
    Unary8(Dest8),
    Unary16(Dest16),
    Binary8(Dest8, Src8),    
    Binary16(Dest16, Src16),
}

pub type OpCode = u32;
pub type Size = usize;
pub type Cycles = usize;

#[derive(Debug, Eq, PartialEq)]
pub struct Inst {
    opcode: OpCode,
    mnemo: Mnemo,
    ops: Operands,
    size: Size,
    cycles: Cycles,
}

macro_rules! inst {
    (ADD HL, BC) => (Inst{opcode: 0x09, mnemo: Mnemo::ADD, ops: Operands::Binary16(Dest::Reg(Reg16::HL), Src::Reg(Reg16::BC)), size: 1, cycles: 11});
    (DEC B) => (Inst{opcode: 0x05, mnemo: Mnemo::DEC, ops: Operands::Unary8(Dest::Reg(Reg8::B)), size: 1, cycles: 4});
    (DEC C) => (Inst{opcode: 0x0d, mnemo: Mnemo::DEC, ops: Operands::Unary8(Dest::Reg(Reg8::C)), size: 1, cycles: 4});
    (DEC BC) => (Inst{opcode: 0x0b, mnemo: Mnemo::DEC, ops: Operands::Unary16(Dest::Reg(Reg16::BC)), size: 1, cycles: 6});
    (EX AF, AF_) => (Inst{opcode: 0x08, mnemo: Mnemo::EX, ops: Operands::Unary16(Dest::Reg(Reg16::AF)), size: 1, cycles: 4});
    (INC B) => (Inst{opcode: 0x04, mnemo: Mnemo::INC, ops: Operands::Unary8(Dest::Reg(Reg8::B)), size: 1, cycles: 4});
    (INC C) => (Inst{opcode: 0x0c, mnemo: Mnemo::INC, ops: Operands::Unary8(Dest::Reg(Reg8::C)), size: 1, cycles: 4});
    (INC BC) => (Inst{opcode: 0x03, mnemo: Mnemo::INC, ops: Operands::Unary16(Dest::Reg(Reg16::BC)), size: 1, cycles: 6});
    (LD A, (BC)) => (Inst{opcode: 0x0a, mnemo: Mnemo::LD, ops: Operands::Binary8(Dest::Reg(Reg8::A), Src::IndReg(Reg16::BC)), size: 1, cycles: 7});
    (LD (BC), A) => (Inst{opcode: 0x02, mnemo: Mnemo::LD, ops: Operands::Binary8(Dest::IndReg(Reg16::BC), Src::Reg(Reg8::A)), size: 1, cycles: 7});
    (LD B, $x:expr) => (Inst{opcode: 0x06, mnemo: Mnemo::LD, ops: Operands::Binary8(Dest::Reg(Reg8::B), Src::Liter($x)), size: 2, cycles: 7});
    (LD C, $x:expr) => (Inst{opcode: 0x0e, mnemo: Mnemo::LD, ops: Operands::Binary8(Dest::Reg(Reg8::C), Src::Liter($x)), size: 2, cycles: 7});
    (LD BC, $x:expr) => (Inst{opcode: 0x01, mnemo: Mnemo::LD, ops: Operands::Binary16(Dest::Reg(Reg16::BC), Src::Liter($x)), size: 3, cycles: 10});
    (NOP) => (Inst{opcode: 0x00, mnemo: Mnemo::NOP, ops: Operands::Nulary, size: 1, cycles: 4});
    (RLCA) => (Inst{opcode: 0x07, mnemo: Mnemo::RLCA, ops: Operands::Nulary, size: 1, cycles: 4});
    (RRCA) => (Inst{opcode: 0x0f, mnemo: Mnemo::RRCA, ops: Operands::Nulary, size: 1, cycles: 4});
}

type DecodeFn = Fn(&mut io::Read) -> io::Result<Inst>;

pub struct Decoder {
    main: Vec<Box<DecodeFn>>,
}

impl Decoder {
    pub fn new() -> Decoder {
        Decoder { main: Self::build_main_table() }
    }

    pub fn decode<R: io::Read>(&self, input: &mut R) -> io::Result<Inst> {
        let opcode = input.read_u8()? as usize;
        self.main[opcode](input)
    }

    fn build_main_table() -> Vec<Box<DecodeFn>> {
        vec! {
            /* 0x00 */ Box::new(|_| { Ok(inst!(NOP)) }),
            /* 0x01 */ Box::new(|r| { Ok(inst!(LD BC, r.read_u16::<LittleEndian>()?)) }),
            /* 0x02 */ Box::new(|_| { Ok(inst!(LD (BC), A)) }),
            /* 0x03 */ Box::new(|_| { Ok(inst!(INC BC)) }),
            /* 0x04 */ Box::new(|_| { Ok(inst!(INC B)) }),
            /* 0x05 */ Box::new(|_| { Ok(inst!(DEC B)) }),
            /* 0x06 */ Box::new(|r| { Ok(inst!(LD B, r.read_u8()?)) }),
            /* 0x07 */ Box::new(|_| { Ok(inst!(RLCA)) }),
            /* 0x08 */ Box::new(|_| { Ok(inst!(EX AF, AF_)) }),
            /* 0x09 */ Box::new(|_| { Ok(inst!(ADD HL, BC)) }),
            /* 0x0a */ Box::new(|_| { Ok(inst!(LD A, (BC))) }),
            /* 0x0b */ Box::new(|_| { Ok(inst!(DEC BC)) }),
            /* 0x0c */ Box::new(|_| { Ok(inst!(INC C)) }),
            /* 0x0d */ Box::new(|_| { Ok(inst!(DEC C)) }),
            /* 0x0e */ Box::new(|r| { Ok(inst!(LD C, r.read_u8()?)) }),
            /* 0x0f */ Box::new(|_| { Ok(inst!(RRCA)) }),            
        }
    }
}

impl Inst {    
    pub fn exec<C: Context>(&self, ctx: &mut C) -> Cycles {
        match self {
            Inst{mnemo: Mnemo::ADD, ops: Operands::Binary8(dst, src), .. } => self.exec_add(ctx, dst, src),
            Inst{mnemo: Mnemo::ADD, ops: Operands::Binary16(dst, src), .. } => self.exec_add(ctx, dst, src),
            Inst{mnemo: Mnemo::DEC, ops: Operands::Unary8(dst), .. } => self.exec_dec(ctx, dst),
            Inst{mnemo: Mnemo::DEC, ops: Operands::Unary16(dst), .. } => self.exec_dec(ctx, dst),
            Inst{mnemo: Mnemo::EX, ops: Operands::Unary8(_), .. } => self.exec_exaf(ctx),
            Inst{mnemo: Mnemo::INC, ops: Operands::Unary8(dst), .. } => self.exec_inc(ctx, dst),
            Inst{mnemo: Mnemo::INC, ops: Operands::Unary16(dst), .. } => self.exec_inc(ctx, dst),
            Inst{mnemo: Mnemo::LD, ops: Operands::Binary16(dst, src), .. } => self.exec_load(ctx, dst, src),
            Inst{mnemo: Mnemo::NOP, .. } => self.exec_nop(ctx),
            Inst{mnemo: Mnemo::RLCA, .. } => self.exec_rlca(ctx),
            Inst{mnemo: Mnemo::RRCA, .. } => self.exec_rrca(ctx),
            _ => unimplemented!("cannot execute illegal instruction"),
        }    
    }

    fn exec_add<C: Context, D: Data>(&self, ctx: &mut C, dst: &Dest<D>, src: &Src<D>) -> Cycles {
        let a = src.read(ctx);
        let b = dst.read(ctx);
        dst.write(ctx, a + b);
        ctx.regs_mut().inc_pc(self.size);
        self.cycles
    }

    fn exec_nop<C: Context>(&self, ctx: &mut C) -> Cycles {
        ctx.regs_mut().inc_pc(self.size);
        self.cycles
    }

    fn exec_inc<C: Context, D: Data>(&self, ctx: &mut C, dst: &Dest<D>) -> Cycles {
        let val = dst.read(ctx);
        dst.write(ctx, D::inc(val));
        ctx.regs_mut().inc_pc(self.size);
        self.cycles
    }

    fn exec_dec<C: Context, D: Data>(&self, ctx: &mut C, dst: &Dest<D>) -> Cycles {
        let val = dst.read(ctx);
        dst.write(ctx, D::dec(val));
        ctx.regs_mut().inc_pc(self.size);
        self.cycles
    }

    fn exec_exaf<C: Context>(&self, ctx: &mut C) -> Cycles {
        ctx.regs_mut().swap_af();
        ctx.regs_mut().inc_pc(self.size);
        self.cycles
    }

    fn exec_load<C: Context, D: Data>(&self, ctx: &mut C, dst: &Dest<D>, src: &Src<D>) -> Cycles {
        let val = src.read(ctx);
        dst.write(ctx, val);
        ctx.regs_mut().inc_pc(self.size);
        self.cycles
    }

    fn exec_rlca<C: Context>(&self, ctx: &mut C) -> Cycles {
        let orig = Reg8::A.read(ctx.regs());
        let dest = (orig << 1) | (orig >> 7);
        Reg8::A.write(ctx.regs_mut(), dest);
        self.cycles
    }

    fn exec_rrca<C: Context>(&self, ctx: &mut C) -> Cycles {
        let orig = Reg8::A.read(ctx.regs());
        let dest = (orig >> 1) | (orig << 7);
        Reg8::A.write(ctx.regs_mut(), dest);
        self.cycles
    }

}

#[cfg(test)]
mod test {
    use cpu::z80::regs::{Reg16};
    use super::*;

    #[test]
    fn should_encode() {
        let tests = [
            EncodeTest {
                what: "nop",
                input: vec![0x00],
                expected: inst!(NOP),
            },
            EncodeTest {
                what: "load bc,1234h",
                input: vec![0x01, 0x34, 0x12],
                expected: inst!(LD BC, 0x1234), 
            },
            EncodeTest {
                what: "load (bc),a",
                input: vec![0x02],
                expected: inst!(LD (BC), A), 
            },
            EncodeTest {
                what: "inc bc",
                input: vec![0x03],
                expected: inst!(INC BC), 
            },
            EncodeTest {
                what: "inc b",
                input: vec![0x04],
                expected: inst!(INC B), 
            },
            EncodeTest {
                what: "dec b",
                input: vec![0x05],
                expected: inst!(DEC B), 
            },
            EncodeTest {
                what: "ld b,12h",
                input: vec![0x06, 0x12],
                expected: inst!(LD B, 0x12), 
            },
            EncodeTest {
                what: "rlca",
                input: vec![0x07],
                expected: inst!(RLCA), 
            },
            EncodeTest {
                what: "ex af,af'",
                input: vec![0x08],
                expected: inst!(EX AF, AF_), 
            },
            EncodeTest {
                what: "add hl,bc'",
                input: vec![0x09],
                expected: inst!(ADD HL, BC), 
            },
            EncodeTest {
                what: "ld a,(bc)'",
                input: vec![0x0a],
                expected: inst!(LD A, (BC)), 
            },
            EncodeTest {
                what: "dec bc'",
                input: vec![0x0b],
                expected: inst!(DEC BC), 
            },
            EncodeTest {
                what: "inc c'",
                input: vec![0x0c],
                expected: inst!(INC C), 
            },
            EncodeTest {
                what: "dec c'",
                input: vec![0x0d],
                expected: inst!(DEC C), 
            },
            EncodeTest {
                what: "ld c,12h'",
                input: vec![0x0e, 0x12],
                expected: inst!(LD C, 0x12), 
            },
            EncodeTest {
                what: "rrca",
                input: vec![0x0f],
                expected: inst!(RRCA), 
            },
        ];
        for test in &tests {
            test.run();
        }
    }

    struct EncodeTest {
        what: &'static str,
        input: Vec<u8>,
        expected: Inst,
    }

    impl EncodeTest {
        fn run(&self) {
            let decoder = Decoder::new();
            let mut read: &[u8] = &self.input;
            let given = decoder.decode(&mut read).unwrap();
            assert_eq!(self.expected, given, "decoding instruction:Dest {}", self.what);
        }
    }
}