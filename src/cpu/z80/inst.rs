pub type OpCode = u32;
pub type Size = usize;
pub type Cycles = usize;

#[cfg(target_endian = "little")]
macro_rules! encode_literal {
    ($v:expr => 0) => (($v & 0x00ff) as u8);
    ($v:expr => 1) => ((($v & 0xff00) >> 8) as u8);
}

macro_rules! inst {
    (ADD HL, BC)        => ([0x09]);
    (DEC B)             => ([0x05]);
    (DEC C)             => ([0x0d]);
    (DEC D)             => ([0x15]);
    (DEC BC)            => ([0x0b]);
    (DJNZ $x:expr)      => ([0x10, $x]);
    (EX AF, AF_)        => ([0x08]);
    (INC B)             => ([0x04]);
    (INC C)             => ([0x0c]);
    (INC D)             => ([0x14]);
    (INC BC)            => ([0x03]);
    (INC DE)            => ([0x13]);
    (JP $x:expr)        => ([0xc3, encode_literal!($x => 0), encode_literal!($x => 1)]);
    (LD A, (BC))        => ([0x0a]);
    (LD B, $x:expr)     => ([0x06, $x]);
    (LD C, $x:expr)     => ([0x0e, $x]);
    (LD D, $x:expr)     => ([0x16, $x]);
    (LD BC, $x:expr)    => ([0x01, encode_literal!($x => 0), encode_literal!($x => 1)]);
    (LD DE, $x:expr)    => ([0x11, encode_literal!($x => 0), encode_literal!($x => 1)]);
    (LD (BC), A)        => ([0x02]);
    (LD (DE), A)        => ([0x12]);
    (NOP)               => ([0x00]);
    (RLA)               => ([0x17]);
    (RLCA)              => ([0x07]);
    (RRCA)              => ([0x0f]);
}
