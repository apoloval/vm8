use std::io;

use byteorder::{ReadBytesExt, LittleEndian};

use bus::{Addr16, Memory16, MemoryItem};
use cpu::z80::data::{Data, Word, Byte};
use cpu::z80::regs::{Reg8, Reg16, Register, Registers};
use cpu::z80::props::{InstProps, InstTime};

// Context trait defines a context where instructions are executed
pub trait Context {
    type Mem: Memory16;
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
#[derive(Debug, PartialEq)]
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
                let addr = Addr16::from(r.read(c.regs()) as u16);
                D::Value::mem_read(c.mem(), addr)
            },
        }
    }
}

// Dest defines a destination operand of a instruction
#[derive(Debug, PartialEq)]
pub enum Dest<D: Data> {
    Reg(D::Reg),
    IndReg(Reg16),
}

impl<D: Data> Dest<D> {
    fn read<C: Context>(&self, c: &C) -> D::Value {
        match self {
            Dest::Reg(r) => r.read(c.regs()),
            Dest::IndReg(r) => {
                let addr = Addr16::from(r.read(c.regs()) as u16);
                D::Value::mem_read(c.mem(), addr)
            },
        }
    }

    fn write<C: Context>(&self, c: &mut C, val: D::Value) {
        match self {
            Dest::Reg(r) => r.write(c.regs_mut(), val),
            Dest::IndReg(r) => {
                let addr = Addr16::from(r.read(c.regs()) as u16);
                D::Value::mem_write(c.mem_mut(), addr, val)
            },
        }
    }
}

type Dest8 = Dest<Byte>;
type Dest16 = Dest<Word>;

#[derive(Debug, PartialEq)]
pub enum Inst {
    ADD16(Dest16, Src16),
    NOP,
    DEC8(Dest8),
    DEC16(Dest16),
    EXAF,
    INC8(Dest8),
    INC16(Dest16),
    LD8(Dest8, Src8),
    LD16(Dest16, Src16),
    RLCA,
    RRCA,
}

impl Inst {
    pub fn decode<R: io::Read>(input: &mut R) -> io::Result<Inst> {
        let opcode = input.read_u8()?;
        let inst = match opcode {
            0x00 => Inst::NOP,
            0x01 => Inst::LD16(
                Dest::Reg(Reg16::BC), 
                Src::Liter(input.read_u16::<LittleEndian>()?),
            ), 
            0x02 => Inst::LD8(
                Dest::IndReg(Reg16::BC), 
                Src::Reg(Reg8::A),
            ), 
            0x03 => Inst::INC16(
                Dest::Reg(Reg16::BC), 
            ), 
            0x04 => Inst::INC8(
                Dest::Reg(Reg8::B), 
            ), 
            0x05 => Inst::DEC8(
                Dest::Reg(Reg8::B), 
            ), 
            0x06 => Inst::LD8(
                Dest::Reg(Reg8::B), 
                Src::Liter(input.read_u8()?), 
            ), 
            0x07 => Inst::RLCA, 
            0x08 => Inst::EXAF, 
            0x09 => Inst::ADD16(
                Dest::Reg(Reg16::HL),
                Src::Reg(Reg16::BC),
            ), 
            0x0a => Inst::LD8(
                Dest::Reg(Reg8::A),
                Src::IndReg(Reg16::BC),
            ), 
            0x0b => Inst::DEC16(
                Dest::Reg(Reg16::BC), 
            ), 
            0x0c => Inst::INC8(
                Dest::Reg(Reg8::C), 
            ), 
            0x0d => Inst::DEC8(
                Dest::Reg(Reg8::C), 
            ), 
            0x0e => Inst::LD8(
                Dest::Reg(Reg8::C),
                Src::Liter(input.read_u8()?),
            ), 
            0x0f => Inst::RRCA, 
            _ => unimplemented!("decoding of given opcode is not implemented"),
        };
        Ok(inst)
    }

    pub fn props(&self) -> InstProps {
        InstProps::from_inst(self)
    }

    pub fn exec<C: Context>(&self, ctx: &mut C) -> InstTime {
        match self {
            Inst::ADD16(dst, src) => self.exec_add(ctx, dst, src),
            Inst::NOP => self.exec_nop(ctx),
            Inst::DEC8(dst) => self.exec_dec(ctx, dst),
            Inst::DEC16(dst) => self.exec_dec(ctx, dst),
            Inst::EXAF => self.exec_exaf(ctx),
            Inst::INC8(dst) => self.exec_inc(ctx, dst),
            Inst::INC16(dst) => self.exec_inc(ctx, dst),
            Inst::LD8(dst, src) => self.exec_load(ctx, dst, src),
            Inst::LD16(dst, src) => self.exec_load(ctx, dst, src),
            Inst::RLCA => self.exec_rlca(ctx),
            Inst::RRCA => self.exec_rrca(ctx),
        }
    }

    fn exec_add<C: Context, D: Data>(&self, ctx: &mut C, dst: &Dest<D>, src: &Src<D>) -> InstTime {
        let props = self.props();
        let a = src.read(ctx);
        let b = dst.read(ctx);
        dst.write(ctx, a + b);
        ctx.regs_mut().inc_pc(props.size);
        props.time
    }

    fn exec_nop<C: Context>(&self, ctx: &mut C) -> InstTime {
        let props = self.props();
        ctx.regs_mut().inc_pc(props.size);
        props.time
    }

    fn exec_inc<C: Context, D: Data>(&self, ctx: &mut C, dst: &Dest<D>) -> InstTime {
        let props = self.props();
        let val = dst.read(ctx);
        dst.write(ctx, D::inc(val));
        ctx.regs_mut().inc_pc(props.size);
        props.time
    }

    fn exec_dec<C: Context, D: Data>(&self, ctx: &mut C, dst: &Dest<D>) -> InstTime {
        let props = self.props();
        let val = dst.read(ctx);
        dst.write(ctx, D::dec(val));
        ctx.regs_mut().inc_pc(props.size);
        props.time
    }

    fn exec_exaf<C: Context>(&self, ctx: &mut C) -> InstTime {
        let props = self.props();
        ctx.regs_mut().swap_af();
        ctx.regs_mut().inc_pc(props.size);
        props.time
    }

    fn exec_load<C: Context, D: Data>(&self, ctx: &mut C, dst: &Dest<D>, src: &Src<D>) -> InstTime {
        let props = self.props();
        let val = src.read(ctx);
        dst.write(ctx, val);
        ctx.regs_mut().inc_pc(props.size);
        props.time
    }

    fn exec_rlca<C: Context>(&self, ctx: &mut C) -> InstTime {
        let props = self.props();
        let orig = Reg8::A.read(ctx.regs());
        let dest = (orig << 1) | (orig >> 7);
        Reg8::A.write(ctx.regs_mut(), dest);
        props.time
    }

    fn exec_rrca<C: Context>(&self, ctx: &mut C) -> InstTime {
        let props = self.props();
        let orig = Reg8::A.read(ctx.regs());
        let dest = (orig >> 1) | (orig << 7);
        Reg8::A.write(ctx.regs_mut(), dest);
        props.time
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
                expected: Inst::NOP,
            },
            EncodeTest {
                what: "load bc,1234h",
                input: vec![0x01, 0x34, 0x12],
                expected: Inst::LD16(
                    Dest::Reg(Reg16::BC), 
                    Src::Liter(0x1234),
                ), 
            },
            EncodeTest {
                what: "load (bc),a",
                input: vec![0x02],
                expected: Inst::LD8(
                    Dest::IndReg(Reg16::BC), 
                    Src::Reg(Reg8::A),
                ), 
            },
            EncodeTest {
                what: "inc bc",
                input: vec![0x03],
                expected: Inst::INC16(
                    Dest::Reg(Reg16::BC), 
                ), 
            },
            EncodeTest {
                what: "inc b",
                input: vec![0x04],
                expected: Inst::INC8(
                    Dest::Reg(Reg8::B), 
                ), 
            },
            EncodeTest {
                what: "dec b",
                input: vec![0x05],
                expected: Inst::DEC8(
                    Dest::Reg(Reg8::B), 
                ), 
            },
            EncodeTest {
                what: "ld b,12h",
                input: vec![0x06, 0x12],
                expected: Inst::LD8(
                    Dest::Reg(Reg8::B), 
                    Src::Liter(0x12), 
                ), 
            },
            EncodeTest {
                what: "rlca",
                input: vec![0x07],
                expected: Inst::RLCA, 
            },
            EncodeTest {
                what: "ex af,af'",
                input: vec![0x08],
                expected: Inst::EXAF, 
            },
            EncodeTest {
                what: "add hl,bc'",
                input: vec![0x09],
                expected: Inst::ADD16(
                    Dest::Reg(Reg16::HL),
                    Src::Reg(Reg16::BC),
                ), 
            },
            EncodeTest {
                what: "ld a,(bc)'",
                input: vec![0x0a],
                expected: Inst::LD8(
                    Dest::Reg(Reg8::A),
                    Src::IndReg(Reg16::BC),
                ), 
            },
            EncodeTest {
                what: "dec bc'",
                input: vec![0x0b],
                expected: Inst::DEC16(
                    Dest::Reg(Reg16::BC),
                ), 
            },
            EncodeTest {
                what: "inc c'",
                input: vec![0x0c],
                expected: Inst::INC8(
                    Dest::Reg(Reg8::C),
                ), 
            },
            EncodeTest {
                what: "dec c'",
                input: vec![0x0d],
                expected: Inst::DEC8(
                    Dest::Reg(Reg8::C),
                ), 
            },
            EncodeTest {
                what: "ld c,12h'",
                input: vec![0x0e, 0x12],
                expected: Inst::LD8(
                    Dest::Reg(Reg8::C),
                    Src::Liter(0x12),
                ), 
            },
            EncodeTest {
                what: "rrca",
                input: vec![0x0f],
                expected: Inst::RRCA, 
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
            let mut read: &[u8] = &self.input;
            let given = Inst::decode(&mut read).unwrap();
            assert_eq!(self.expected, given, "decoding instruction:Dest {}", self.what);
        }
    }
}