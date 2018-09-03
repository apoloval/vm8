#![cfg_attr(all(feature = "nightly", test), feature(test))]

#[cfg(all(feature = "nightly", test))]
extern crate test;

extern crate byteorder;
extern crate num_traits;

#[cfg(test)]
extern crate rand;

pub mod bus;
pub mod clock;
pub mod cpu;
pub mod mem;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
