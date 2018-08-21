pub type OpCode = u32;
pub type Size = usize;
pub type Cycles = usize;

#[derive(Debug, Eq, PartialEq)]
pub struct Inst {
    pub opcode: OpCode,
    pub extra8: u8,
    pub extra16: u16,
    pub size: Size,
    pub cycles: Cycles,
}

macro_rules! inst {
    (ADD HL, BC)        => ($crate::cpu::z80::Inst{opcode: 0x09, extra8: 00, extra16: 00, size: 1, cycles: 11});
    (DEC B)             => ($crate::cpu::z80::Inst{opcode: 0x05, extra8: 00, extra16: 00, size: 1, cycles: 4});
    (DEC C)             => ($crate::cpu::z80::Inst{opcode: 0x0d, extra8: 00, extra16: 00, size: 1, cycles: 4});
    (DEC BC)            => ($crate::cpu::z80::Inst{opcode: 0x0b, extra8: 00, extra16: 00, size: 1, cycles: 6});
    (EX AF, AF_)        => ($crate::cpu::z80::Inst{opcode: 0x08, extra8: 00, extra16: 00, size: 1, cycles: 4});
    (INC B)             => ($crate::cpu::z80::Inst{opcode: 0x04, extra8: 00, extra16: 00, size: 1, cycles: 4});
    (INC C)             => ($crate::cpu::z80::Inst{opcode: 0x0c, extra8: 00, extra16: 00, size: 1, cycles: 4});
    (INC BC)            => ($crate::cpu::z80::Inst{opcode: 0x03, extra8: 00, extra16: 00, size: 1, cycles: 6});
    (JP $x:expr)        => ($crate::cpu::z80::Inst{opcode: 0xc3, extra8: 00, extra16: $x, size: 3, cycles: 10});
    (LD A, (BC))        => ($crate::cpu::z80::Inst{opcode: 0x0a, extra8: 00, extra16: 00, size: 1, cycles: 7});
    (LD (BC), A)        => ($crate::cpu::z80::Inst{opcode: 0x02, extra8: 00, extra16: 00, size: 1, cycles: 7});
    (LD B, $x:expr)     => ($crate::cpu::z80::Inst{opcode: 0x06, extra8: $x, extra16: 00, size: 2, cycles: 7});
    (LD C, $x:expr)     => ($crate::cpu::z80::Inst{opcode: 0x0e, extra8: $x, extra16: 00, size: 2, cycles: 7});
    (LD BC, $x:expr)    => ($crate::cpu::z80::Inst{opcode: 0x01, extra8: 00, extra16: $x, size: 3, cycles: 10});
    (NOP)               => ($crate::cpu::z80::Inst{opcode: 0x00, extra8: 00, extra16: 00, size: 1, cycles: 4});
    (RLCA)              => ($crate::cpu::z80::Inst{opcode: 0x07, extra8: 00, extra16: 00, size: 1, cycles: 4});
    (RRCA)              => ($crate::cpu::z80::Inst{opcode: 0x0f, extra8: 00, extra16: 00, size: 1, cycles: 4});
}
