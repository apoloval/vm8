use std::boxed::Box;
use num_traits::{Bounded, Num};

/// An address is any type that can be used to identify a bus target
pub trait Address : Copy + Num + Bounded + Into<usize> {}

impl Address for u8 {}
impl Address for u16 {}

/// A data is any type that be used to represented exchanged information
pub trait Data : Copy + Num + Default {}

impl Data for u8 {}
impl Data for u16 {}

/// A bus is a channel that can be used to communicate devices.
///
/// The bus is operated via read and write operations using an address to identify the target.
/// Both the data exchanged and the addresses are defined by the implementation. Please note
/// a memory system is a special case of a bus. In such case, each memory location can be seen
/// as a independent device.
pub trait Bus<A: Address, D: Data> {
    /// Read data from the given address.
    fn read_from(&mut self, addr: A) -> D;

    /// Write data to the given address.
    fn write_to(&mut self, addr: A, val: D);
}

impl<T, A, D> Bus<A, D> for Box<T> where T: Bus<A, D> + ?Sized, A: Address, D: Data {
    fn read_from(&mut self, addr: A) -> D {
        (**self).read_from(addr)
    }

    fn write_to(&mut self, addr: A, val: D) {
        (**self).write_to(addr, val)
    }
}

/// A dead bus is a bus that does not respond to read or write operations.
///
/// For read operations, it returns the default value for the data type used in the bus. For write
/// actions, it simply ignores the operation. It only makes sense for testing or development only.
pub struct Dead;

impl<A: Address, D: Data> Bus<A, D> for Dead {
    fn read_from(&mut self, _addr: A) -> D { D::default() }
    fn write_to(&mut self, _addr: A, _val: D) {}
}
