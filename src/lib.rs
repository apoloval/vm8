#![cfg_attr(all(feature = "nightly", test), feature(test))]
#![cfg_attr(feature = "nightly", feature(trace_macros))]

#[cfg(all(feature = "nightly", test))]
extern crate test;

extern crate byteorder;
extern crate num_traits;
extern crate time;

#[cfg(test)]
extern crate rand;

#[cfg(test)]
#[macro_use]
pub mod testutil;

pub mod bus;
pub mod clock;
pub mod cpu;
pub mod io;
pub mod mem;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
