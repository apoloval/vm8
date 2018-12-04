macro_rules! flag {
    (S, $flags:expr)    => (($flags & 0x80) >> 7);
    (Z, $flags:expr)    => (($flags & 0x40) >> 6);
    (H, $flags:expr)    => (($flags & 0x10) >> 4);
    (PV, $flags:expr)   => (($flags & 0x04) >> 2);
    (N, $flags:expr)    => (($flags & 0x02) >> 1);
    (C, $flags:expr)    => ($flags & 0x01);

    (NZ, $flags:expr)   => (flag!(Z, $flags) ^ 1);
    (NC, $flags:expr)   => (flag!(C, $flags) ^ 1);
}

macro_rules! flags_bitmask_set {
    (C)         => (0b00000001);
    (N)         => (0b00000010);
    (PV)        => (0b00000100);
    (H)         => (0b00010000);
    (Z)         => (0b01000000);
    (S)         => (0b10000000);
    ($($a:ident),+) => ($(flags_bitmask_reset!($a))|+);
}

macro_rules! flags_bitmask_reset {
    (C)         => (0b11111110);
    (N)         => (0b11111101);
    (PV)        => (0b11111011);
    (H)         => (0b11101111);
    (Z)         => (0b10111111);
    (S)         => (0b01111111);
    ($($a:ident),+) => ($(flags_bitmask_reset!($a))&+);
}

macro_rules! flags_apply {
    ($a:expr, ) => ($a);
    ($a:expr, $f:ident:0 $($rest:tt)*) => (flags_apply!($a & flags_bitmask_reset!($f), $($rest)*));
    ($a:expr, $f:ident:1 $($rest:tt)*) => (flags_apply!($a | flags_bitmask_set!($f), $($rest)*));
    ($a:expr, $f:ident:[$c:expr] $($rest:tt)*) => (flags_apply!((if $c { $a | flags_bitmask_set!($f) } else { $a & flags_bitmask_reset!($f) }) as u8, $($rest)*));
    ($a:expr, [$($f:ident),+]:[$c:expr] $($rest:tt)*) => (flags_apply!(if $c { $a | flags_bitmask_set!($($f),+) } else { $a & flags_bitmask_reset!($($f),+) }, $($rest)*));
}
