use std::ops::Add;

pub trait Address: Copy + Sized + Add<Output=Self> + Add<usize,Output=Self> {}

impl<T: Copy + Add<Output=Self> + Add<usize,Output=Self>> Address for T {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Addr8(u8);

impl Add for Addr8 {
    type Output = Addr8;

    fn add(self, rhs: Addr8) -> Addr8 {
        Addr8(self.0 + rhs.0)
    }
}

impl From<u8> for Addr8 {
    fn from(val: u8) -> Addr8 {
        Addr8(val)
    }
}

impl Add<usize> for Addr8 {
    type Output = Addr8;

    fn add(self, rhs: usize) -> Addr8 {
        Addr8(self.0 + rhs as u8)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Addr16(u16);

impl Add for Addr16 {
    type Output = Addr16;

    fn add(self, rhs: Addr16) -> Addr16 {
        Addr16(self.0 + rhs.0)
    }
}

impl Add<usize> for Addr16 {
    type Output = Addr16;

    fn add(self, rhs: usize) -> Addr16 {
        Addr16(self.0 + rhs as u16)
    }
}

impl From<u16> for Addr16 {
    fn from(val: u16) -> Addr16 {
        Addr16(val)
    }
}

impl From<Addr16> for u16 {
    fn from(val: Addr16) -> u16 {
        val.0
    }
}
