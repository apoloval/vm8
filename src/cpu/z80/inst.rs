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
    (ADD HL, SP)        => ([0x39]);
    (CCF)               => ([0x3f]);
    (CPL)               => ([0x2f]);
    (DAA)               => ([0x27]);
    (DEC A)             => ([0x3d]);
    (DEC B)             => ([0x05]);
    (DEC C)             => ([0x0d]);
    (DEC D)             => ([0x15]);
    (DEC E)             => ([0x1d]);
    (DEC H)             => ([0x25]);
    (DEC L)             => ([0x2d]);
    (DEC BC)            => ([0x0b]);
    (DEC DE)            => ([0x1b]);
    (DEC HL)            => ([0x2b]);
    (DEC SP)            => ([0x3b]);
    (DEC (HL))          => ([0x35]);
    (DJNZ $x:expr)      => ([0x10, $x]);
    (EX AF, AF_)        => ([0x08]);
    (INC A)             => ([0x3c]);
    (INC B)             => ([0x04]);
    (INC C)             => ([0x0c]);
    (INC D)             => ([0x14]);
    (INC E)             => ([0x1c]);
    (INC H)             => ([0x24]);
    (INC L)             => ([0x2c]);
    (INC BC)            => ([0x03]);
    (INC DE)            => ([0x13]);
    (INC HL)            => ([0x23]);
    (INC SP)            => ([0x33]);
    (INC (HL))          => ([0x34]);
    (JP $x:expr)        => ([0xc3, encode_literal!($x => 0), encode_literal!($x => 1)]);
    (JR $x:expr)        => ([0x18, $x]);
    (JR C, $x:expr)     => ([0x38, $x]);
    (JR NC, $x:expr)    => ([0x30, $x]);
    (JR NZ, $x:expr)    => ([0x20, $x]);
    (JR Z, $x:expr)     => ([0x28, $x]);
    (LD A, (BC))        => ([0x0a]);
    (LD A, (DE))        => ([0x1a]);
    (LD A, ($x:expr))   => ([0x3a, encode_literal!($x => 0), encode_literal!($x => 1)]);
    (LD A, $x:expr)     => ([0x3e, $x]);
    (LD B, A)           => ([0x47]);
    (LD B, B)           => ([0x40]);
    (LD B, C)           => ([0x41]);
    (LD B, D)           => ([0x42]);
    (LD B, E)           => ([0x43]);
    (LD B, H)           => ([0x44]);
    (LD B, L)           => ([0x45]);
    (LD B, (HL))        => ([0x46]);
    (LD B, $x:expr)     => ([0x06, $x]);
    (LD C, A)           => ([0x4f]);
    (LD C, B)           => ([0x48]);
    (LD C, C)           => ([0x49]);
    (LD C, D)           => ([0x4a]);
    (LD C, E)           => ([0x4b]);
    (LD C, H)           => ([0x4c]);
    (LD C, L)           => ([0x4d]);
    (LD C, (HL))        => ([0x4e]);
    (LD D, A)           => ([0x57]);
    (LD D, B)           => ([0x50]);
    (LD D, C)           => ([0x51]);
    (LD D, D)           => ([0x52]);
    (LD D, E)           => ([0x53]);
    (LD D, H)           => ([0x54]);
    (LD D, L)           => ([0x55]);
    (LD D, (HL))        => ([0x56]);
    (LD E, A)           => ([0x5f]);
    (LD E, B)           => ([0x58]);
    (LD E, C)           => ([0x59]);
    (LD E, D)           => ([0x5a]);
    (LD E, E)           => ([0x5b]);
    (LD E, H)           => ([0x5c]);
    (LD E, L)           => ([0x5d]);
    (LD E, (HL))        => ([0x5e]);
    (LD C, $x:expr)     => ([0x0e, $x]);
    (LD D, $x:expr)     => ([0x16, $x]);
    (LD E, $x:expr)     => ([0x1e, $x]);
    (LD H, A)           => ([0x67]);
    (LD H, B)           => ([0x60]);
    (LD H, C)           => ([0x61]);
    (LD H, D)           => ([0x62]);
    (LD H, E)           => ([0x63]);
    (LD H, H)           => ([0x64]);
    (LD H, L)           => ([0x65]);
    (LD H, (HL))        => ([0x66]);
    (LD H, $x:expr)     => ([0x26, $x]);
    (LD L, A)           => ([0x6f]);
    (LD L, B)           => ([0x68]);
    (LD L, C)           => ([0x69]);
    (LD L, D)           => ([0x6a]);
    (LD L, E)           => ([0x6b]);
    (LD L, H)           => ([0x6c]);
    (LD L, L)           => ([0x6d]);
    (LD L, (HL))        => ([0x6e]);
    (LD L, $x:expr)     => ([0x2e, $x]);
    (LD BC, $x:expr)    => ([0x01, encode_literal!($x => 0), encode_literal!($x => 1)]);
    (LD DE, $x:expr)    => ([0x11, encode_literal!($x => 0), encode_literal!($x => 1)]);
    (LD HL, ($x:expr))  => ([0x2a, encode_literal!($x => 0), encode_literal!($x => 1)]);
    (LD HL, $x:expr)    => ([0x21, encode_literal!($x => 0), encode_literal!($x => 1)]);
    (LD SP, $x:expr)    => ([0x31, encode_literal!($x => 0), encode_literal!($x => 1)]);
    (LD (BC), A)        => ([0x02]);
    (LD (DE), A)        => ([0x12]);
    (LD (HL), B)        => ([0x70]);
    (LD (HL), C)        => ([0x71]);
    (LD (HL), D)        => ([0x72]);
    (LD (HL), E)        => ([0x73]);
    (LD (HL), H)        => ([0x74]);
    (LD (HL), $x:expr)  => ([0x36, $x]);
    (LD ($x:expr), A)   => ([0x32, encode_literal!($x => 0), encode_literal!($x => 1)]);
    (LD ($x:expr), HL)  => ([0x22, encode_literal!($x => 0), encode_literal!($x => 1)]);
    (NOP)               => ([0x00]);
    (RLA)               => ([0x17]);
    (RLCA)              => ([0x07]);
    (RRA)               => ([0x1f]);
    (RRCA)              => ([0x0f]);
    (SCF)               => ([0x37]);
}
