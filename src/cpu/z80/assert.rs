macro_rules! assert_r8 {
    ($cpu:expr, $reg:ident, $expected:expr) => ({
        let actual = cpu_eval!($cpu, $reg);
        assert_result!(HEX8, stringify!($reg), $expected, actual);
    })
}

macro_rules! assert_r16 {
    ($cpu:expr, $reg:ident, $expected:expr) => ({
        let actual = cpu_eval!($cpu, $reg);
        assert_result!(HEX16, stringify!($reg), $expected, actual);
    })
}

macro_rules! assert_flags {
    ($cpu:expr, $f0:expr, ($($flags:tt)+)) => ({
        let initial = $f0;
        let expected = flags_apply!(initial, $($flags)+);
        let actual = cpu_eval!($cpu, F);
        assert_result!(BIN8, "flags", expected, actual);
    });
    (unaffected, $cpu:expr, $f0:expr) => ({
        let expected = $f0;
        let actual = cpu_eval!($cpu, F);
        assert_result!(BIN8, "flags", expected, actual);
    });
    ($cpu:expr, $expected:expr, $f0:expr) => ({
        let initial = $f0;
        let expected = $expected(initial);
        let actual = cpu_eval!($cpu, F);
        assert_result!(BIN8, "flags", expected, actual);
    });
}

macro_rules! assert_program_counter {
    ($cpu:expr, $expected:expr) => ({
        let actual = cpu_eval!($cpu, PC);
        assert_result!(HEX16, "program counter", $expected, actual);
    });
}

macro_rules! assert_dest {
    ($type:ty, $cpu:expr, $dstget:expr, $expected:expr) => ({
        let actual = $dstget($cpu);
        assert_result!(HEX16, "dest", $expected, actual);
    });
}
