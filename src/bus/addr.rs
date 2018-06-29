use std::ops::Add;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Address(usize);

impl Add for Address {
    type Output = Address;

    fn add(self, rhs: Address) -> Address {
        Address(self.0 + rhs.0)
    }
}

impl Add<usize> for Address {
    type Output = Address;

    fn add(self, rhs: usize) -> Address {
        Address(self.0 + rhs)
    }
}

impl From<u16> for Address {
    fn from(val: u16) -> Address {
        Address(val as usize)
    }
}

impl From<Address> for u16 {
    fn from(val: Address) -> u16 {
        val.0 as u16
    }
}

impl From<Address> for usize {
    fn from(val: Address) -> usize {
        val.0 as usize
    }
}
