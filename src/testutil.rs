macro_rules! assert_result {
    (BIN8, $pre:expr, $a:expr, $b:expr) => (
        assert_eq!($a, $b, "{} expects {:08b}b but {:08b}b given ", $pre, $a, $b)
    );
    (HEX8, $pre:expr, $a:expr, $b:expr) => (
        assert_eq!($a, $b, "{} expects {:02x}h but {:02x}h given ", $pre, $a, $b)
    );
    (HEX16, $pre:expr, $a:expr, $b:expr) => (
        assert_eq!($a, $b, "{} expects {:04x}h but {:04x}h given ", $pre, $a, $b)
    );
}

macro_rules! table_test {
    ($cases:expr, $body:expr) => {
        for case in $cases {
            print!("Test case '{}': ", case.name);
            $body(case);
            println!("OK");
        }
    }
}