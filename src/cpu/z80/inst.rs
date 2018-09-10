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
    (ADD HL, HL)        => ([0x29]);
    (ADD HL, DE)        => ([0x19]);
    (CPL)               => ([0x2f]);
    (DAA)               => ([0x27]);
    (DEC B)             => ([0x05]);
    (DEC C)             => ([0x0d]);
    (DEC D)             => ([0x15]);
    (DEC E)             => ([0x1d]);
    (DEC H)             => ([0x25]);
    (DEC L)             => ([0x2d]);
    (DEC BC)            => ([0x0b]);
    (DEC DE)            => ([0x1b]);
    (DEC HL)            => ([0x2b]);
    (DJNZ $x:expr)      => ([0x10, $x]);
    (EX AF, AF_)        => ([0x08]);
    (INC B)             => ([0x04]);
    (INC C)             => ([0x0c]);
    (INC D)             => ([0x14]);
    (INC E)             => ([0x1c]);
    (INC H)             => ([0x24]);
    (INC L)             => ([0x2c]);
    (INC BC)            => ([0x03]);
    (INC DE)            => ([0x13]);
    (INC HL)            => ([0x23]);
    (JP $x:expr)        => ([0xc3, encode_literal!($x => 0), encode_literal!($x => 1)]);
    (JR $x:expr)        => ([0x18, $x]);
    (JR NC, $x:expr)    => ([0x30, $x]);
    (JR NZ, $x:expr)    => ([0x20, $x]);
    (JR Z, $x:expr)     => ([0x28, $x]);
    (LD A, (BC))        => ([0x0a]);
    (LD A, (DE))        => ([0x1a]);
    (LD B, $x:expr)     => ([0x06, $x]);
    (LD C, $x:expr)     => ([0x0e, $x]);
    (LD D, $x:expr)     => ([0x16, $x]);
    (LD E, $x:expr)     => ([0x1e, $x]);
    (LD H, $x:expr)     => ([0x26, $x]);
    (LD L, $x:expr)     => ([0x2e, $x]);
    (LD BC, $x:expr)    => ([0x01, encode_literal!($x => 0), encode_literal!($x => 1)]);
    (LD DE, $x:expr)    => ([0x11, encode_literal!($x => 0), encode_literal!($x => 1)]);
    (LD HL, ($x:expr))  => ([0x2a, encode_literal!($x => 0), encode_literal!($x => 1)]);
    (LD HL, $x:expr)    => ([0x21, encode_literal!($x => 0), encode_literal!($x => 1)]);
    (LD (BC), A)        => ([0x02]);
    (LD (DE), A)        => ([0x12]);
    (LD ($x:expr), HL)  => ([0x22, encode_literal!($x => 0), encode_literal!($x => 1)]);
    (NOP)               => ([0x00]);
    (RLA)               => ([0x17]);
    (RLCA)              => ([0x07]);
    (RRA)               => ([0x1f]);
    (RRCA)              => ([0x0f]);
}
