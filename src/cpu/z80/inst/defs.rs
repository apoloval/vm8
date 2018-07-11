macro_rules! inst {
    (ADD HL, BC) => (Inst{opcode: 0x09, mnemo: Mnemo::ADD, ops: Operands::Binary16(Dest::Reg(reg::Name16::HL), Src::Reg(reg::Name16::BC)), size: 1, cycles: 11});
    (DEC B) => (Inst{opcode: 0x05, mnemo: Mnemo::DEC, ops: Operands::UnaryDest8(Dest::Reg(reg::Name8::B)), size: 1, cycles: 4});
    (DEC C) => (Inst{opcode: 0x0d, mnemo: Mnemo::DEC, ops: Operands::UnaryDest8(Dest::Reg(reg::Name8::C)), size: 1, cycles: 4});
    (DEC BC) => (Inst{opcode: 0x0b, mnemo: Mnemo::DEC, ops: Operands::UnaryDest16(Dest::Reg(reg::Name16::BC)), size: 1, cycles: 6});
    (EX AF, AF_) => (Inst{opcode: 0x08, mnemo: Mnemo::EX, ops: Operands::UnaryDest16(Dest::Reg(reg::Name16::AF)), size: 1, cycles: 4});
    (INC B) => (Inst{opcode: 0x04, mnemo: Mnemo::INC, ops: Operands::UnaryDest8(Dest::Reg(reg::Name8::B)), size: 1, cycles: 4});
    (INC C) => (Inst{opcode: 0x0c, mnemo: Mnemo::INC, ops: Operands::UnaryDest8(Dest::Reg(reg::Name8::C)), size: 1, cycles: 4});
    (INC BC) => (Inst{opcode: 0x03, mnemo: Mnemo::INC, ops: Operands::UnaryDest16(Dest::Reg(reg::Name16::BC)), size: 1, cycles: 6});
    (JP $x:expr) => (Inst{opcode: 0xc3, mnemo: Mnemo::JP, ops: Operands::UnarySrc16(Src::Liter($x)), size: 3, cycles: 10});
    (LD A, (BC)) => (Inst{opcode: 0x0a, mnemo: Mnemo::LD, ops: Operands::Binary8(Dest::Reg(reg::Name8::A), Src::IndReg(reg::Name16::BC)), size: 1, cycles: 7});
    (LD (BC), A) => (Inst{opcode: 0x02, mnemo: Mnemo::LD, ops: Operands::Binary8(Dest::IndReg(reg::Name16::BC), Src::Reg(reg::Name8::A)), size: 1, cycles: 7});
    (LD B, $x:expr) => (Inst{opcode: 0x06, mnemo: Mnemo::LD, ops: Operands::Binary8(Dest::Reg(reg::Name8::B), Src::Liter($x)), size: 2, cycles: 7});
    (LD C, $x:expr) => (Inst{opcode: 0x0e, mnemo: Mnemo::LD, ops: Operands::Binary8(Dest::Reg(reg::Name8::C), Src::Liter($x)), size: 2, cycles: 7});
    (LD BC, $x:expr) => (Inst{opcode: 0x01, mnemo: Mnemo::LD, ops: Operands::Binary16(Dest::Reg(reg::Name16::BC), Src::Liter($x)), size: 3, cycles: 10});
    (NOP) => (Inst{opcode: 0x00, mnemo: Mnemo::NOP, ops: Operands::Nulary, size: 1, cycles: 4});
    (RLCA) => (Inst{opcode: 0x07, mnemo: Mnemo::RLCA, ops: Operands::Nulary, size: 1, cycles: 4});
    (RRCA) => (Inst{opcode: 0x0f, mnemo: Mnemo::RRCA, ops: Operands::Nulary, size: 1, cycles: 4});
}
