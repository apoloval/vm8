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

macro_rules! flags_apply_add8 {
    ($flags:expr, $a:expr, $b:expr, $r:expr) => (flags_apply!($flags,
            S:[($r & 0x00ff) >= 0x80]
            Z:[$r == 0]
            H:[($a & 0x0f) + ($b & 0x0f) >= 0x10]
            PV:[$before < 0x80 && $after >= 0x80]
            N:0
            C:[$r > 0xff]))
}