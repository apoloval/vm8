pub type OpCode = u32;
pub type Size = usize;
pub type Cycles = usize;

#[cfg(target_endian = "little")]
macro_rules! u16_bytes {
    ($v:expr) => (($v & 0x00ff), ($v & 0xff00 >> 8))
}

macro_rules! inst {
    (ADD HL, BC)        => ([0x09]);
    (DEC B)             => ([0x05]);
    (DEC C)             => ([0x0d]);
    (DEC BC)            => ([0x0b]);
    (EX AF, AF_)        => ([0x08]);
    (INC B)             => ([0x04]);
    (INC C)             => ([0x0c]);
    (INC BC)            => ([0x03]);
    (JP $x:expr)        => ([0xc3, u16_bytes!($x)]);
    (LD A, (BC))        => ([0x0a]);
    (LD (BC), A)        => ([0x02]);
    (LD B, $x:expr)     => ([0x06, $x]);
    (LD C, $x:expr)     => ([0x0e, $x]);
    (LD BC, $x:expr)    => ([0x01, $x]);
    (NOP)               => ([0x00]);
    (RLCA)              => ([0x07]);
    (RRCA)              => ([0x0f]);
}
