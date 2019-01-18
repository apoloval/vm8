use rand;

/// Declare a test function.
/// This is a shortcut for declaring a nulary function and mark it as `#[test]`.
macro_rules! decl_test {
        ($fname:ident, $body:block) => {
            #[test]
            fn $fname() {
                $body
            }
        };
    }

/// Declare a test suite.
/// This is a shortcut for declaring a mod and importing all definitions from its parent.
macro_rules! decl_suite {
    ($sname:ident, { $($items:item)+ }) => {
        mod $sname {
            use super::*;

            $($items)+
        }
    };
}

/// Declare a test suite.
/// An alias of `decl_scenario!()`
macro_rules! decl_scenario {
    ($sname:ident, { $($items:item)+ }) => {
        decl_suite!($sname, { $($items)+ });
    };
}

/// A type that can produce a sample value of itself.
pub trait Sample {
        fn sample() -> Self;
    }

impl Sample for u8 {
    fn sample() -> u8 { rand::random() }
}

impl Sample for u16 {
    fn sample() -> u16 { rand::random() }
}

/// Assert the result with a given representation matches the expected value
macro_rules! assert_result {
    (BIN8, $pre:expr, $expected:expr, $given:expr) => (
        assert_eq!($expected, $given,
            "{} expects {:08b}b but {:08b}b given ", $pre, $expected, $given)
    );
    (HEX8, $pre:expr, $expected:expr, $given:expr) => (
        assert_eq!($expected, $given,
            "{} expects {:02X}h but {:02X}h given ", $pre, $expected, $given)
    );
    (HEX16, $pre:expr, $expected:expr, $given:expr) => (
        assert_eq!($expected, $given,
            "{} expects {:04X}h but {:04X}h given ", $pre, $expected, $given)
    );
}
