use byteorder::ByteOrder;
use num_traits::{Bounded, Num, One};

// An address is any type that behaves like a number
pub trait Address : Copy + Num + Bounded + Into<usize> {}
impl<T> Address for T where T: Copy + Num + Bounded + Into<usize> {}

// A bus is a channel that can be used to communicate devices.
//
// The bus is operated via read and write operations using an address to identify the target.
// Both the data exchanged and the addresses are defined by the implementation. Please note
// a memory system is a special case of a bus. In such case, each memory location can be seen
// as a independent device. 
pub trait Bus {
    type Addr: Address;
    type Data;

    // Read data at the given address.
    fn read_from(&self, addr: Self::Addr) -> Self::Data;

    // Write data into the given address.
    fn write_to(&mut self, addr: Self::Addr, val: Self::Data);
}

// Read operations for 8-bits buses
pub trait ReadFromBytes : Bus<Data=u8> {
    // Read a word from the bus by fetching two subsequent bytes
    fn read_word_from<O: ByteOrder>(&self, addr: Self::Addr) -> u16 {
        let data = [self.read_from(addr), self.read_from(addr + Self::Addr::one())];
        O::read_u16(&data)
    }
}
impl<T> ReadFromBytes for T where T: Bus<Data=u8> {}

// Write operations for 8-bits buses
pub trait WriteFromBytes : Bus<Data=u8> {
    // Write a word to the bus by sending two subsequent bytes
    fn write_word_to<O: ByteOrder>(&mut self, addr: Self::Addr, val: u16) {
        let mut data = [0; 2];
        O::write_u16(&mut data, val);
        self.write_to(addr, data[0]);
        self.write_to(addr + Self::Addr::one(), data[1]);
    }
}
impl<T> WriteFromBytes for T where T: Bus<Data=u8> {}

// Read a 16-bits word from a 8-bits bus
pub fn read_word_from<O, B>(bus: &B, addr: B::Addr) -> u16 
where B: Bus<Data=u8>, O: ByteOrder {
    let data = [bus.read_from(addr), bus.read_from(addr + B::Addr::one())];
    O::read_u16(&data)
}

// Write a 16-bits word into a 8-bits bus
pub fn write_word_to<O, B>(bus: &mut B, addr: B::Addr, val: u16) 
where B: Bus<Data=u8>, O: ByteOrder {
    let mut data = [0; 2];
    O::write_u16(&mut data, val);
    bus.write_to(addr, data[0]);
    bus.write_to(addr + B::Addr::one(), data[1]);
}
