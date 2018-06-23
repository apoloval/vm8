extern crate byteorder;
extern crate num_traits;

pub mod bus;
pub mod cpu;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
