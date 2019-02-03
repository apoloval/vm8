use std::io;
use std::marker::PhantomData;

use byteorder::ByteOrder;

use crate::bus;
use crate::bus::{Address, Data, Bus};

/// Read operations for 8-bits memory buses
pub trait ReadFromBytes<A: Address> : Bus<A, u8> {
    /// Read a word from the bus by fetching two subsequent bytes
    fn read_word_from_mem<O: ByteOrder>(&mut self, addr: A) -> u16 {
        let data = [self.read_from(addr), self.read_from(addr + A::one())];
        O::read_u16(&data)
    }
}
impl<T, A> ReadFromBytes<A> for T where T: Bus<A, u8>, A: Address {}

/// Write operations for 8-bits memory buses
pub trait WriteFromBytes<A: Address> : Bus<A, u8> {
    /// Write a word to the bus by sending two subsequent bytes
    fn write_word_to_mem<O: ByteOrder>(&mut self, addr: A, val: u16) {
        let mut data = [0; 2];
        O::write_u16(&mut data, val);
        self.write_to(addr, data[0]);
        self.write_to(addr + A::one(), data[1]);
    }
}

impl<T, A> WriteFromBytes<A> for T where T: Bus<A, u8>, A: Address {}

pub struct MemoryBank<A: Address, D: Data> {
    address: PhantomData<A>,
    data: Vec<D>,
}

impl<A: bus::Address, D: Data> MemoryBank<A, D> {
    pub fn new() -> Self {
        let address = PhantomData;
        let size: usize = A::max_value().into() - A::min_value().into() + 1;
        let data = vec![D::default(); size];
        Self { address, data }
    }

    pub fn data(&self) -> &[D] { self.data.as_slice() }
}

impl<A: bus::Address> MemoryBank<A, u8> {
    pub fn from_data<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let mut bank = Self::new();
        reader.read(&mut bank.data).map(|_| bank)
    }

    pub fn copy_to<R: io::Read>(&mut self, addr: A, reader: &mut R) -> io::Result<usize> {
        let dest = &mut self.data[A::into(addr)..];
        reader.read(dest)
    }
}

impl<A: Address, D: Data> bus::Bus<A, D> for MemoryBank<A, D> {
    fn read_from(&mut self, addr: A) -> D {
        let offset: usize = addr.into();
        self.data[offset]
    }

    fn write_to(&mut self, addr: A, val: D) {
        let offset: usize = addr.into();
        self.data[offset] = val;
    }
}

impl<A: bus::Address> io::Write for MemoryBank<A, u8> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut buff: &mut [u8] = &mut self.data;
        buff.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.data.flush()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::bus::Bus;

    decl_test!(test_memory_bank_copy, {
        let mut bank = MemoryBank::<u16, u8>::new();
        let mut data: &[u8] = &[1u8, 2, 3, 4];
        let result = bank.copy_to(0x4000, &mut data);
        assert_eq!(result.unwrap(), 4);
        assert_eq!(1, bank.read_from(0x4000));
        assert_eq!(2, bank.read_from(0x4001));
        assert_eq!(3, bank.read_from(0x4002));
        assert_eq!(4, bank.read_from(0x4003));
    });
}
