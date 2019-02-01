use std::boxed::Box;
use num_traits::{Bounded, Num};

/// An address is any type that behaves like a number
pub trait Address : Copy + Num + Bounded + Into<usize> {}
impl<T> Address for T where T: Copy + Num + Bounded + Into<usize> {}

/// A bus is a channel that can be used to communicate devices.
///
/// The bus is operated via read and write operations using an address to identify the target.
/// Both the data exchanged and the addresses are defined by the implementation. Please note
/// a memory system is a special case of a bus. In such case, each memory location can be seen
/// as a independent device.
pub trait Bus {
    type Addr: Address;
    type Data;

    /// Read data from the given address.
    fn read_from(&self, addr: Self::Addr) -> Self::Data;

    /// Write data to the given address.
    fn write_to(&mut self, addr: Self::Addr, val: Self::Data);
}

impl<T> Bus for Box<T> where T: Bus + ?Sized {
    type Addr = <T as Bus>::Addr;
    type Data = <T as Bus>::Data;

    fn read_from(&self, addr: Self::Addr) -> Self::Data {
        (**self).read_from(addr)
    }

    fn write_to(&mut self, addr: Self::Addr, val: Self::Data) {
        (**self).write_to(addr, val)
    }
}

/// A dead bus is a bus that does not respond to read or write operations.
///
/// For read operations, it returns the default value for the data type used in the bus. For write
/// actions, it simply ignores the operation. It only makes sense for testing or development only.
pub struct Dead<A: Address, D: Default> {
    m1: std::marker::PhantomData<A>,
    m2: std::marker::PhantomData<D>,
}

impl<A: Address, D: Default> Dead<A, D> {
    pub fn new() -> Self {
        Dead {
            m1: std::marker::PhantomData,
            m2: std::marker::PhantomData,
        }
    }
}

impl<A: Address, D: Default> Bus for Dead<A, D> {
    type Addr = A;
    type Data = D;

    fn read_from(&self, _addr: Self::Addr) -> Self::Data {
        D::default()
    }

    fn write_to(&mut self, _addr: Self::Addr, _val: Self::Data) {}
}
