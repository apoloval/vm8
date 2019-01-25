macro_rules! flag {
    ($flags:expr, S)    => (($flags & 0x80) >> 7);
    ($flags:expr, Z)    => (($flags & 0x40) >> 6);
    ($flags:expr, H)    => (($flags & 0x10) >> 4);
    ($flags:expr, PV)   => (($flags & 0x04) >> 2);
    ($flags:expr, N)    => (($flags & 0x02) >> 1);
    ($flags:expr, C)    => ($flags & 0x01);

    ($flags:expr, NZ)   => (flag!($flags, Z) ^ 1);
    ($flags:expr, NC)   => (flag!($flags, C) ^ 1);
}

macro_rules! flags_bitmask_set {
    ($a:expr, C)         => ($a | 0b00000001);
    ($a:expr, N)         => ($a | 0b00000010);
    ($a:expr, PV)        => ($a | 0b00000100);
    ($a:expr, H)         => ($a | 0b00010000);
    ($a:expr, Z)         => ($a | 0b01000000);
    ($a:expr, S)         => ($a | 0b10000000);

    ($a:expr, NC)        => (flags_bitmask_reset!($a, C));
    ($a:expr, NZ)        => (flags_bitmask_reset!($a, Z));
}

macro_rules! flags_bitmask_reset {
    ($a:expr, C)         => ($a & 0b11111110);
    ($a:expr, N)         => ($a & 0b11111101);
    ($a:expr, PV)        => ($a & 0b11111011);
    ($a:expr, H)         => ($a & 0b11101111);
    ($a:expr, Z)         => ($a & 0b10111111);
    ($a:expr, S)         => ($a & 0b01111111);

    ($a:expr, NC)        => (flags_bitmask_set!($a, C));
    ($a:expr, NZ)        => (flags_bitmask_set!($a, Z));
}

macro_rules! flags_apply {
    ($a:expr, ) => ($a);
    ($a:expr, $f:ident:0 $($rest:tt)*) => (
        flags_apply!(flags_bitmask_reset!($a, $f), $($rest)*)
    );
    ($a:expr, $f:ident:1 $($rest:tt)*) => (
        flags_apply!(flags_bitmask_set!($a, $f), $($rest)*)
    );
    ($a:expr, $f:ident:[$c:expr] $($rest:tt)*) => (
        flags_apply!(
            (if $c { flags_bitmask_set!($a, $f) } else { flags_bitmask_reset!($a, $f) }) as u8,
            $($rest)*)
    );
}
