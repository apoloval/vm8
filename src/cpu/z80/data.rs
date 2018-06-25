use byteorder::{ByteOrder, LittleEndian};
use num_traits::{Num, One};

use bus::{MemoryItem};
use cpu::z80::regs::{Reg8, Reg16, Register};

pub trait Data {
    type Ord: ByteOrder;
    type Value: Num + MemoryItem<Self::Ord> + Copy;
    type Reg: Register<Self::Value>;

    fn inc(v: Self::Value) -> Self::Value { v + Self::Value::one() }
}

#[derive(Debug, PartialEq)]
pub struct Byte;

impl Data for Byte {
    type Ord = LittleEndian;
    type Value = u8;
    type Reg = Reg8;
}

#[derive(Debug, PartialEq)]
pub struct Word;

impl Data for Word {
    type Ord = LittleEndian;
    type Value = u16;
    type Reg = Reg16;
}
