use std::io;

use bus::BurstRead;
use cpu::z80::inst::Inst;

type DecodeFn = fn(&mut BurstRead) -> io::Result<Inst>;

pub struct Decoder {
    main: Vec<DecodeFn>,
}

impl Decoder {
    pub fn new() -> Decoder {
        Decoder { main: Self::build_main_table() }
    }

    pub fn decode<R: BurstRead>(&self, input: &mut R) -> io::Result<Inst> {
        let opcode = input.read_byte();
        self.main[opcode as usize](input)
    }

    fn build_main_table() -> Vec<DecodeFn> {
        vec! {
            /* 0x00 */ |_| { Ok(inst!(NOP)) },
            /* 0x01 */ |r| { Ok(inst!(LD BC, r.read_word())) },
            /* 0x02 */ |_| { Ok(inst!(LD (BC), A)) },
            /* 0x03 */ |_| { Ok(inst!(INC BC)) },
            /* 0x04 */ |_| { Ok(inst!(INC B)) },
            /* 0x05 */ |_| { Ok(inst!(DEC B)) },
            /* 0x06 */ |r| { Ok(inst!(LD B, r.read_byte())) },
            /* 0x07 */ |_| { Ok(inst!(RLCA)) },
            /* 0x08 */ |_| { Ok(inst!(EX AF, AF_)) },
            /* 0x09 */ |_| { Ok(inst!(ADD HL, BC)) },
            /* 0x0a */ |_| { Ok(inst!(LD A, (BC))) },
            /* 0x0b */ |_| { Ok(inst!(DEC BC)) },
            /* 0x0c */ |_| { Ok(inst!(INC C)) },
            /* 0x0d */ |_| { Ok(inst!(DEC C)) },
            /* 0x0e */ |r| { Ok(inst!(LD C, r.read_byte())) },
            /* 0x0f */ |_| { Ok(inst!(RRCA)) },
            /* 0x10 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x11 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x12 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x13 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x14 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x15 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x16 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x17 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x18 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x19 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x1a */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x1b */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x1c */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x1d */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x1e */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x1f */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x20 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x21 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x22 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x23 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x24 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x25 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x26 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x27 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x28 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x29 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x2a */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x2b */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x2c */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x2d */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x2e */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x2f */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x30 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x31 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x32 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x33 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x34 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x35 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x36 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x37 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x38 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x39 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x3a */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x3b */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x3c */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x3d */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x3e */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x3f */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x40 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x41 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x42 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x43 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x44 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x45 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x46 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x47 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x48 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x49 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x4a */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x4b */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x4c */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x4d */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x4e */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x4f */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x50 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x51 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x52 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x53 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x54 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x55 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x56 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x57 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x58 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x59 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x5a */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x5b */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x5c */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x5d */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x5e */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x5f */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x60 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x61 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x62 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x63 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x64 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x65 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x66 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x67 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x68 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x69 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x6a */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x6b */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x6c */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x6d */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x6e */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x6f */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x70 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x71 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x72 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x73 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x74 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x75 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x76 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x77 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x78 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x79 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x7a */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x7b */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x7c */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x7d */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x7e */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x7f */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x80 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x81 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x82 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x83 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x84 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x85 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x86 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x87 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x88 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x89 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x8a */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x8b */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x8c */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x8d */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x8e */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x8f */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x90 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x91 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x92 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x93 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x94 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x95 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x96 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x97 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x98 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x99 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x9a */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x9b */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x9c */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x9d */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x9e */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0x9f */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xa0 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xa1 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xa2 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xa3 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xa4 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xa5 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xa6 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xa7 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xa8 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xa9 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xaa */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xab */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xac */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xad */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xae */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xaf */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xb0 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xb1 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xb2 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xb3 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xb4 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xb5 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xb6 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xb7 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xb8 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xb9 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xba */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xbb */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xbc */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xbd */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xbe */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xbf */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xc0 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xc1 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xc2 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xc3 */ |r| { Ok(inst!(JP r.read_word())) },
            /* 0xc4 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xc5 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xc6 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xc7 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xc8 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xc9 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xca */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xcb */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xcc */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xcd */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xce */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xcf */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xd0 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xd1 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xd2 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xd3 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xd4 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xd5 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xd6 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xd7 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xd8 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xd9 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xda */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xdb */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xdc */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xdd */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xde */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xdf */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xe0 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xe1 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xe2 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xe3 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xe4 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xe5 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xe6 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xe7 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xe8 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xe9 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xea */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xeb */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xec */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xed */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xee */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xef */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xf0 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xf1 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xf2 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xf3 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xf4 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xf5 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xf6 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xf7 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xf8 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xf9 */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xfa */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xfb */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xfc */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xfd */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xfe */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },
            /* 0xff */ |_| { Err(io::Error::from(io::ErrorKind::InvalidInput)) },        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_encode() {
        let tests = [
            DecodeTest {
                what: "NOP",
                input: vec![0x00],
                expected: inst!(NOP),
            },
            DecodeTest {
                what: "LD BC,1234h",
                input: vec![0x01, 0x34, 0x12],
                expected: inst!(LD BC, 0x1234), 
            },
            DecodeTest {
                what: "LD (BC),A",
                input: vec![0x02],
                expected: inst!(LD (BC), A), 
            },
            DecodeTest {
                what: "INC BC",
                input: vec![0x03],
                expected: inst!(INC BC), 
            },
            DecodeTest {
                what: "INC B",
                input: vec![0x04],
                expected: inst!(INC B), 
            },
            DecodeTest {
                what: "DEC B",
                input: vec![0x05],
                expected: inst!(DEC B), 
            },
            DecodeTest {
                what: "LD B,12h",
                input: vec![0x06, 0x12],
                expected: inst!(LD B, 0x12), 
            },
            DecodeTest {
                what: "RLCA",
                input: vec![0x07],
                expected: inst!(RLCA), 
            },
            DecodeTest {
                what: "EX AF,AF'",
                input: vec![0x08],
                expected: inst!(EX AF, AF_), 
            },
            DecodeTest {
                what: "ADD HL,BC'",
                input: vec![0x09],
                expected: inst!(ADD HL, BC), 
            },
            DecodeTest {
                what: "LD A,(BC)'",
                input: vec![0x0a],
                expected: inst!(LD A, (BC)), 
            },
            DecodeTest {
                what: "DEC BC'",
                input: vec![0x0b],
                expected: inst!(DEC BC), 
            },
            DecodeTest {
                what: "INC C'",
                input: vec![0x0c],
                expected: inst!(INC C), 
            },
            DecodeTest {
                what: "DEC C'",
                input: vec![0x0d],
                expected: inst!(DEC C), 
            },
            DecodeTest {
                what: "LD C,12h'",
                input: vec![0x0e, 0x12],
                expected: inst!(LD C, 0x12), 
            },
            DecodeTest {
                what: "RRCA",
                input: vec![0x0f],
                expected: inst!(RRCA), 
            },
            DecodeTest {
                what: "JP 1234h",
                input: vec![0xc3, 0x34, 0x12],
                expected: inst!(JP 0x1234), 
            },
        ];
        for test in &tests {
            test.run();
        }
    }

    struct DecodeTest {
        what: &'static str,
        input: Vec<u8>,
        expected: Inst,
    }

    impl DecodeTest {
        fn run(&self) {
            let decoder = Decoder::new();
            let mut read: &[u8] = &self.input;
            let given = decoder.decode(&mut read).unwrap();
            assert_eq!(self.expected, given, "decoding instruction:Dest {}", self.what);
        }
    }
}
