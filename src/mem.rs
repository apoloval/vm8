use std::io;
use std::marker::PhantomData;

use bus;

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
}

impl<A: bus::Address> bus::Bus for MemoryBank<A> {
    type Addr = A;
    type Data = u8;

    fn read(&self, addr: A) -> u8 {
        let offset: usize = addr.into();
        self.data[offset]
    }

    fn write(&mut self, addr: A, val: u8) {
        let offset: usize = addr.into();
        self.data[offset] = val;
    }
}
