use std::io;
use std::marker::PhantomData;

use crate::bus;

pub struct MemoryBank<A: bus::Address> {
    address: PhantomData<A>,
    data: Vec<u8>,
}

impl<A: bus::Address> MemoryBank<A> {
    pub fn new() -> Self {
        let address = PhantomData;
        let size: usize = A::max_value().into() + 1;
        let data = vec![0; size];
        Self { address, data }
    }

    pub fn from_data<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let mut bank = Self::new();
        reader.read(&mut bank.data).map(|_| bank)
    }

    pub fn copy_to<R: io::Read>(&mut self, addr: A, reader: &mut R) -> io::Result<usize> {
        let dest = &mut self.data[A::into(addr)..];
        reader.read(dest)
    }

    pub fn data(&self) -> &[u8] { self.data.as_slice() }
}

impl<A: bus::Address> bus::Bus for MemoryBank<A> {
    type Addr = A;
    type Data = u8;

    fn read_from(&self, addr: A) -> u8 {
        let offset: usize = addr.into();
        self.data[offset]
    }

    fn write_to(&mut self, addr: A, val: u8) {
        let offset: usize = addr.into();
        self.data[offset] = val;
    }
}

impl<A: bus::Address> io::Write for MemoryBank<A> {
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
        let mut bank = MemoryBank::<u16>::new();
        let mut data: &[u8] = &[1u8, 2, 3, 4];
        let result = bank.copy_to(0x4000, &mut data);
        assert_eq!(result.unwrap(), 4);
        assert_eq!(1, bank.read_from(0x4000));
        assert_eq!(2, bank.read_from(0x4001));
        assert_eq!(3, bank.read_from(0x4002));
        assert_eq!(4, bank.read_from(0x4003));
    });
}
