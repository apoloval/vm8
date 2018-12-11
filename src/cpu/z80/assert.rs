macro_rules! assert_cpu {
    ($type:ident, $cpu:expr, $eval:tt, $expected:expr) => ({
        let actual = cpu_eval!($cpu, $eval);
        assert_result!($type, stringify!($eval), $expected, actual);
    });
}

macro_rules! assert_r8 {
    ($cpu:expr, $reg:tt, $expected:expr) => ({
        assert_cpu!(HEX8, $cpu, $reg, $expected);
    })
}

macro_rules! assert_r16 {
    ($cpu:expr, $reg:tt, $expected:expr) => ({
        assert_cpu!(HEX16, $cpu, $reg, $expected);
    })
}

macro_rules! assert_flags {
    ($cpu:expr, $f0:expr, unaffected) => ({
        assert_cpu!(BIN8, $cpu, F, $f0);
    });
    ($cpu:expr, $f0:expr, ($($flags:tt)+)) => ({
        let expected = flags_apply!($f0, $($flags)+);
        assert_cpu!(BIN8, $cpu, F, expected);
    });
}

macro_rules! assert_pc {
    ($cpu:expr, $expected:expr) => { 
        assert_cpu!(HEX16, $cpu, PC, $expected)
    };
}
