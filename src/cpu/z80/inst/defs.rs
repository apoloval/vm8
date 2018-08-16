macro_rules! inst {
    (ADD HL, BC)        => (Inst{opcode: 0x09, extra8: 00, extra16: 00, size: 1, cycles: 11});
    (DEC B)             => (Inst{opcode: 0x05, extra8: 00, extra16: 00, size: 1, cycles: 4});
    (DEC C)             => (Inst{opcode: 0x0d, extra8: 00, extra16: 00, size: 1, cycles: 4});
    (DEC BC)            => (Inst{opcode: 0x0b, extra8: 00, extra16: 00, size: 1, cycles: 6});
    (EX AF, AF_)        => (Inst{opcode: 0x08, extra8: 00, extra16: 00, size: 1, cycles: 4});
    (INC B)             => (Inst{opcode: 0x04, extra8: 00, extra16: 00, size: 1, cycles: 4});
    (INC C)             => (Inst{opcode: 0x0c, extra8: 00, extra16: 00, size: 1, cycles: 4});
    (INC BC)            => (Inst{opcode: 0x03, extra8: 00, extra16: 00, size: 1, cycles: 6});
    (JP $x:expr)        => (Inst{opcode: 0xc3, extra8: 00, extra16: $x, size: 3, cycles: 10});
    (LD A, (BC))        => (Inst{opcode: 0x0a, extra8: 00, extra16: 00, size: 1, cycles: 7});
    (LD (BC), A)        => (Inst{opcode: 0x02, extra8: 00, extra16: 00, size: 1, cycles: 7});
    (LD B, $x:expr)     => (Inst{opcode: 0x06, extra8: $x, extra16: 00, size: 2, cycles: 7});
    (LD C, $x:expr)     => (Inst{opcode: 0x0e, extra8: $x, extra16: 00, size: 2, cycles: 7});
    (LD BC, $x:expr)    => (Inst{opcode: 0x01, extra8: 00, extra16: $x, size: 3, cycles: 10});
    (NOP)               => (Inst{opcode: 0x00, extra8: 00, extra16: 00, size: 1, cycles: 4});
    (RLCA)              => (Inst{opcode: 0x07, extra8: 00, extra16: 00, size: 1, cycles: 4});
    (RRCA)              => (Inst{opcode: 0x0f, extra8: 00, extra16: 00, size: 1, cycles: 4});
}
