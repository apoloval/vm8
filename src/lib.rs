#![allow(dead_code)]
#![allow(unused_macros)]

pub mod cpu;
pub mod dev;
pub mod emu;
pub mod io;
pub mod system;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
