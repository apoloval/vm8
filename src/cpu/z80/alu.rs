/// A table with precalculated flags
struct FlagsTable {
    add: Vec<u8>,
    sub: Vec<u8>,
    and: Vec<u8>,
    or: Vec<u8>,
    daa: Vec<u8>,
}

impl FlagsTable {
    /// Returns a flags table with pre-computed values.
    pub fn new() -> Self {
        let mut add_flags = vec![0u8; 256*256];
        let mut sub_flags = vec![0u8; 256*256];
        let mut and_flags = vec![0u8; 256];
        let mut or_flags = vec![0u8; 256];
        let mut daa_flags = vec![0u8; 256];
        for c in 0u8..=255 {
            and_flags[c as usize] = Self::flags_for_bitwise(c, true);
            or_flags[c as usize] = Self::flags_for_bitwise(c, false);
            daa_flags[c as usize] = Self::flags_for_daa(c);
            for a in 0u8..=255 {
                let index = Self::index_of_binops(a, c);
                add_flags[index] = Self::flags_for_add(a, c);
                sub_flags[index] = Self::flags_for_sub(a, c);
            }
        }
        Self {
            add: add_flags,
            sub: sub_flags,
            and: and_flags,
            or: or_flags,
            daa: daa_flags,
        }
    }

    /// Returns the precomputed flags for adding
    #[inline]
    pub fn add_flags(&self, oldval: u8, newval: u8) -> u8 {
        self.add[Self::index_of_binops(oldval, newval)]
    }

    /// Returns the precomputed flags for subtracting
    #[inline]
    pub fn sub_flags(&self, oldval: u8, newval: u8) -> u8 {
        self.sub[Self::index_of_binops(oldval, newval)]
    }

    /// Returns the flags for adding 16-bits numbers.
    ///
    /// Due to memory size limitations and considering 16-bits addition in Z80
    /// is quite slow, these flags are not memoized but computed on the fly.
    #[inline]
    pub fn add16_flags(&self, oldval: u16, newval: u16, flags: &mut u8) {
        let oldh = (oldval >> 8) as u8;
        let newh = (newval >> 8) as u8;
        *flags = flags_apply!(*flags,
            C:[Self::carry_flag(oldval, newval)]
            H:[Self::halfcarry_flag(oldh, newh)]
            N:0);
    }

    /// Returns the precomputed flags for bitwise AND
    #[inline]
    pub fn and_flags(&self, newval: u8) -> u8 {
        self.and[newval as usize]
    }

    /// Returns the precomputed flags for bitwise XOR
    #[inline]
    pub fn or_flags(&self, newval: u8) -> u8 {
        self.or[newval as usize]
    }

    /// Returns the precomputed flags for bitwise XOR
    #[inline]
    pub fn xor_flags(&self, newval: u8) -> u8 {
        // XOR and OR flags are the same
        self.or[newval as usize]
    }

    /// Returns the flags for BCD adjustment.
    ///
    /// The flags table memoizes flags for S, Z and PV. The rest, H, N and C
    /// are calculated on the fly.
    #[inline]
    pub fn bcd_adjust_flags(&self, a: u8, c: u8, cf: u8, hf: u8, nf: u8) -> u8 {
        let h = a >> 4;
        let l = a & 0x0f;
        // Tables obtained from The Undocumented Z80 Documented, page 17.
        // http://datasheets.chipdb.org/Zilog/Z80/z80-documented-0.90.pdf
        let carry = match (cf, h, l) {
            (0, 0x0..=0x9, 0x0..=0x9) => false,
            (0, 0x0..=0x8, 0xa..=0xf) => false,
            (0, 0x9..=0xf, 0xa..=0xf) => true,
            (0, 0xa..=0xf, 0x0..=0x9) => true,
            (1,  _, _) => true,
            _ => unreachable!(),
        };
        let halfcarry = match (nf, hf, l) {
            (0, _, 0x0..=0x9) => false,
            (0, _, 0xa..=0xf) => true,
            (1, 0, _) => false,
            (1, 1, 0x6..=0xf) => false,
            (1, 1, 0x0..=0x5) => true,
            _ => unreachable!(),
        };
        flags_apply!(self.daa[c as usize],
            H:[halfcarry]
            N:[nf != 0]
            C:[carry]
        )
    }

    #[inline]
    fn index_of_binops(a: u8, c: u8) -> usize {
        ((a as usize) << 8) | (c as usize)
    }

    fn flags_for_add(a: u8, c: u8) -> u8 {
        let b = ((c as i16) - (a as i16)) as u8;
        flags_apply!(0,
            S:[Self::signed_flag(c)]
            Z:[Self::zero_flag(c)]
            H:[Self::halfcarry_flag(a, c)]
            PV:[Self::overflow_flag(a, b, c)]
            N:0
            C:[Self::carry_flag(a, c)]
        )
    }

    fn flags_for_sub(a: u8, c: u8) -> u8 {
        let b = ((a as i16) - (c as i16)) as u8;
        flags_apply!(0,
            S:[Self::signed_flag(c)]
            Z:[Self::zero_flag(c)]
            H:[Self::halfborrow_flag(a, c)]
            PV:[Self::underflow_flag(a, b, c)]
            N:1
            C:[Self::borrow_flag(a, c)]
        )
    }

    fn flags_for_bitwise(c: u8, h: bool) -> u8 {
        flags_apply!(0,
            S:[Self::signed_flag(c)]
            Z:[Self::zero_flag(c)]
            H:[h]
            PV:[Self::parity_of(c) % 2 == 0]
            N:0
            C:0
        )
    }

    fn flags_for_daa(c: u8) -> u8 {
        flags_apply!(0,
            S:[Self::signed_flag(c)]
            Z:[Self::zero_flag(c)]
            PV:[Self::parity_of(c) % 2 == 0]
        )
    }

    fn signed_flag(c: u8) -> bool { (c & 0x80) != 0 }
    fn zero_flag(c: u8) -> bool { c == 0 }
    fn halfcarry_flag(a: u8, c: u8) -> bool { (a & 0x0f) > (c & 0x0f) }
    fn halfborrow_flag(a: u8, c: u8) -> bool { (a & 0x0f) < (c & 0x0f) }
    fn overflow_flag(a: u8, b: u8, c: u8) -> bool { (a ^ b ^ 0x80) & (b ^ c) & 0x80 != 0 }
    fn underflow_flag(a: u8, b: u8, c: u8) -> bool { (a ^ b) & ((a ^ c) & 0x80) != 0 }
    fn carry_flag<N: PartialOrd>(a: N, c: N) -> bool { c < a }
    fn borrow_flag<N: PartialOrd>(a: N, c: N) -> bool { c > a }

    fn parity_of(mut n: u8) -> usize {
        let mut parity = 0;
        while n > 0 {
            if n & 0x01 > 0 {
                parity += 1;
            }
            n = n >> 1;
        }
        parity
    }
}

pub struct ALU {
    flags: FlagsTable,
}

impl ALU {
    pub fn new() -> Self {
        Self { flags: FlagsTable::new() }
    }

    #[inline]
    pub fn add8(&self, a: u8, b: u8) -> u8 {
        ((a as u16) + (b as u16)) as u8
    }

    #[inline]
    pub fn adc8(&self, a: u8, b: u8, c: u8) -> u8 {
        if c > 0 {
            ((self.add8(a, b) as u8) + 1) as u8
        } else {
            self.add8(a, b)
        }
    }

    #[inline]
    pub fn add8_with_flags(&self, a: u8, b: u8, flags: &mut u8) -> u8 {
        let c = self.add8(a, b);
        *flags = self.flags.add_flags(a, c);
        c
    }

    #[inline]
    pub fn adc8_with_flags(&self, a: u8, b: u8, flags: &mut u8) -> u8 {
        let c = self.adc8(a, b, flag!(*flags, C));
        *flags = self.flags.add_flags(a, c);
        c
    }

    #[inline]
    pub fn inc8_with_flags(&self, a: u8, flags: &mut u8) -> u8 {
        let c = self.add8(a, 1);
        *flags = (*flags & 0x01) | (self.flags.add_flags(a, c) & 0xfe);
        c
    }

    #[inline]
    pub fn add16(&self, a: u16, b: u16) -> u16 {
        let c = (a as u32) + (b as u32);
        c as u16
    }

    #[inline]
    pub fn add16_with_flags(&self, a: u16, b: u16, flags: &mut u8) -> u16 {
        let c = self.add16(a, b);
        self.flags.add16_flags(a, c, flags);
        c
    }

    #[inline]
    pub fn sub8(&self, a: u8, b: u8) -> u8 {
        let c = (a as i16) - (b as i16);
        c as u8
    }

    #[inline]
    pub fn sbc8(&self, a: u8, b: u8, c: u8) -> u8 {
        if c > 0 {
            ((self.sub8(a, b) as i16) -  1) as u8
        } else {
            self.sub8(a, b)
        }
    }

    #[inline]
    pub fn sub8_with_flags(&self, a: u8, b: u8, flags: &mut u8) -> u8 {
        let c = self.sub8(a, b);
        *flags = self.flags.sub_flags(a, c);
        c
    }

    #[inline]
    pub fn sbc8_with_flags(&self, a: u8, b: u8, flags: &mut u8) -> u8 {
        let c = self.sbc8(a, b, flag!(*flags, C));
        *flags = self.flags.sub_flags(a, c);
        c
    }

    #[inline]
    pub fn dec8_with_flags(&self, a: u8, flags: &mut u8) -> u8 {
        let c = self.sub8(a, 1);
        *flags = (*flags & 0x01) | (self.flags.sub_flags(a, c) & 0xfe);
        c
    }

    #[inline]
    pub fn sub16(&self, a: u16, b: u16) -> u16 {
        ((a as i32) - (b as i32)) as u16
    }

    /// Rotates the given byte 1 bit to the left.
    ///
    /// The previous carry is moved to the rightmost bit. And the previous
    /// leftmost bit is moved to the carry.
    #[inline]
    pub fn rotate_left(&self, val: u8, flags: &mut u8) -> u8 {
        let cf = flag!(*flags, C);
        let carry = val >> 7;
        *flags = flags_apply!(*flags, C:[carry > 0] H:0 N:0);
        (val << 1) | cf
    }

    /// Rotates the given byte 1 bit to the left, with immediate rotation of
    /// the carry.
    ///
    /// The previous carry is discarded. And the previous leftmost bit is moved
    /// to the carry and the rightmost bit.
    #[inline]
    pub fn rotate_left_with_carry(&self, val: u8, flags: &mut u8) -> u8 {
        let carry = val >> 7;
        *flags = flags_apply!(*flags, C:[carry > 0] H:0 N:0);
        (val << 1) | carry
    }

    /// Rotates the given byte 1 bit to the right.
    ///
    /// The previous carry is moved to the leftmost bit. And the previous
    /// rightmost bit is moved to the carry.
    #[inline]
    pub fn rotate_right(&self, val: u8, flags: &mut u8) -> u8 {
        let cf = flag!(*flags, C);
        let carry = val & 0x01;
        *flags = flags_apply!(*flags, C:[carry > 0] H:0 N:0);
        (val >> 1) | (cf << 7)
    }

    /// Rotates the given byte 1 bit to the right, with immediate rotation of
    /// the carry.
    ///
    /// The previous carry is discarded. And the previous rightmost bit is moved
    /// to the carry and the leftmost bit.
    #[inline]
    pub fn rotate_right_with_carry(&self, val: u8, flags: &mut u8) -> u8 {
        let carry = val & 0x01;
        *flags = flags_apply!(*flags, C:[carry > 0] H:0 N:0);
        (val >> 1) | (carry << 7)
    }

    #[inline]
    pub fn bitwise_and(&self, a: u8, b:u8, flags: &mut u8) -> u8 {
        let c = a & b;
        *flags = self.flags.and_flags(c);
        c
    }

    #[inline]
    pub fn bitwise_or(&self, a: u8, b:u8, flags: &mut u8) -> u8 {
        let c = a | b;
        *flags = self.flags.or_flags(c);
        c
    }

    #[inline]
    pub fn bitwise_xor(&self, a: u8, b:u8, flags: &mut u8) -> u8 {
        let c = a ^ b;
        *flags = self.flags.xor_flags(c);
        c
    }

    #[inline]
    pub fn bcd_adjust(&self, a: u8, flags: &mut u8) -> u8 {
        let h = a >> 4;
        let l = a & 0x0f;
        let cf = flag!(*flags, C);
        let hf = flag!(*flags, H);
        let nf = flag!(*flags, N);
        // Table obtained from The Undocumented Z80 Documented, page 17.
        // http://datasheets.chipdb.org/Zilog/Z80/z80-documented-0.90.pdf
        let diff = match (cf, h, hf, l) {
            (0, 0x0..=0x9, 0, 0x0..=0x9) => 0x00,
            (0, 0x0..=0x9, 1, 0x0..=0x9) => 0x06,
            (0, 0x0..=0x8, _, 0xa..=0xf) => 0x06,
            (0, 0xa..=0xf, 0, 0x0..=0x9) => 0x60,
            (1, _,         0, 0x0..=0x9) => 0x60,
            (1, _,         1, 0x0..=0x9) => 0x66,
            (1, _,         _, 0xa..=0xf) => 0x66,
            (0, 0x9..=0xf, _, 0xa..=0xf) => 0x66,
            (0, 0xa..=0xf, 1, 0x0..=0x9) => 0x66,
            _ => unreachable!(),
        };
        let c = if nf == 0 {
            self.add8(a, diff)
        } else {
            self.sub8(a, diff)
        };
        *flags = self.flags.bcd_adjust_flags(a, c, cf, hf, nf);
        c
    }
}

#[cfg(test)]
mod test {
    use super::*;

    decl_scenario!(alu_add8, {
        macro_rules! decl_test_case {
            ($cname:ident, inputs: $a:expr, $b:expr; outputs: $c:expr, ($($flags:tt)+)) => {
                decl_test!($cname, {
                    let alu = ALU::new();
                    let mut flags = 0;
                    let c = alu.add8_with_flags($a, $b, &mut flags);
                    assert_result!(HEX8, "result", $c, c);
                    assert_result!(BIN8, "flags", flags_apply!(0, $($flags)+), flags);
                });
            };
        }

        decl_test_case!(base_case,
            inputs: 0x21, 0x42;
            outputs: 0x63, (S:0 Z:0 H:0 PV:0 N:0 C:0));

        decl_test_case!(signed_flag_set,
            inputs: 0xa0, 0x05;
            outputs: 0xa5, (S:1 Z:0 H:0 PV:0 N:0 C:0));

        decl_test_case!(zero_flag_set,
            inputs: 0x00, 0x00;
            outputs: 0x00, (S:0 Z:1 H:0 PV:0 N:0 C:0));

        decl_test_case!(halfcarry_flag_set,
            inputs: 0x29, 0x38;
            outputs: 0x61, (S:0 Z:0 H:1 PV:0 N:0 C:0));

        decl_test_case!(overflow_flag_set,
            inputs: 0x51, 0x32;
            outputs: 0x83, (S:1 Z:0 H:0 PV:1 N:0 C:0));

        decl_test_case!(carry_flag_set,
            inputs: 0xf0, 0x20;
            outputs: 0x10, (S:0 Z:0 H:0 PV:0 N:0 C:1));
    });

    decl_scenario!(alu_adc8, {
        macro_rules! decl_test_case {
            ($cname:ident, inputs: $a:expr, $b:expr, $cf:expr; outputs: $c:expr, ($($flags:tt)+)) => {
                decl_test!($cname, {
                    let alu = ALU::new();
                    let mut flags = flags_apply!(0, C:[$cf != 0]);
                    let c = alu.adc8_with_flags($a, $b, &mut flags);
                    assert_result!(HEX8, "result", $c, c);
                    assert_result!(BIN8, "flags", flags_apply!(0, $($flags)+), flags);
                });
            };
        }

        decl_test_case!(no_carry,
            inputs: 0x21, 0x42, 0;
            outputs: 0x63, (S:0 Z:0 H:0 PV:0 N:0 C:0));

        decl_test_case!(carry,
            inputs: 0x21, 0x42, 1;
            outputs: 0x64, (S:0 Z:0 H:0 PV:0 N:0 C:0));
    });

    decl_scenario!(alu_sub8, {
        macro_rules! decl_test_case {
            ($cname:ident, inputs: $a:expr, $b:expr; outputs: $c:expr, ($($flags:tt)+)) => {
                decl_test!($cname, {
                    let alu = ALU::new();
                    let mut flags = 0;
                    let c = alu.sub8_with_flags($a, $b, &mut flags);
                    assert_result!(HEX8, "result", $c, c);
                    assert_result!(BIN8, "flags", flags_apply!(0, $($flags)+), flags);
                });
            };
        }

        decl_test_case!(base_case,
            inputs: 0x56, 0x21;
            outputs: 0x35, (S:0 Z:0 H:0 PV:0 N:1 C:0));

        decl_test_case!(signed_flag_set,
            inputs: 0x95, 0x12;
            outputs: 0x83, (S:1 Z:0 H:0 PV:0 N:1 C:0));

        decl_test_case!(zero_flag_set,
            inputs: 0x10, 0x10;
            outputs: 0x00, (S:0 Z:1 H:0 PV:0 N:1 C:0));

        decl_test_case!(halfcarry_flag_set,
            inputs: 0x21, 0x02;
            outputs: 0x1f, (S:0 Z:0 H:1 PV:0 N:1 C:0));

        decl_test_case!(overflow_flag_set,
            inputs: 0x85, 0x10;
            outputs: 0x75, (S:0 Z:0 H:0 PV:1 N:1 C:0));

        decl_test_case!(carry_flag_set,
            inputs: 0x81, 0x90;
            outputs: 0xf1, (S:1 Z:0 H:0 PV:0 N:1 C:1));
    });

    decl_scenario!(alu_sbc8, {
        macro_rules! decl_test_case {
            ($cname:ident, inputs: $a:expr, $b:expr, $cf:expr; outputs: $c:expr, ($($flags:tt)+)) => {
                decl_test!($cname, {
                    let alu = ALU::new();
                    let mut flags = flags_apply!(0, C:[$cf != 0]);
                    let c = alu.sbc8_with_flags($a, $b, &mut flags);
                    assert_result!(HEX8, "result", $c, c);
                    assert_result!(BIN8, "flags", flags_apply!(0, $($flags)+), flags);
                });
            };
        }

        decl_test_case!(no_carry,
            inputs: 0x45, 0x21, 0;
            outputs: 0x24, (S:0 Z:0 H:0 PV:0 N:1 C:0));

        decl_test_case!(carry,
            inputs: 0x45, 0x21, 1;
            outputs: 0x23, (S:0 Z:0 H:0 PV:0 N:1 C:0));
    });

    decl_scenario!(alu_inc8, {
        macro_rules! decl_test_case {
            ($cname:ident, inputs: $a:expr, ($($f0:tt)+); outputs: $c:expr, ($($flags:tt)+)) => {
                decl_test!($cname, {
                    let alu = ALU::new();
                    let initial_flags = flags_apply!(0, $($f0)+);
                    let mut flags = initial_flags;
                    let c = alu.inc8_with_flags($a, &mut flags);
                    assert_result!(HEX8, "result", $c, c);
                    assert_result!(BIN8, "flags", flags_apply!(initial_flags, $($flags)+), flags);
                });
            };
        }

        decl_test_case!(base_case,
            inputs: 0x21, (C:0);
            outputs: 0x22, (S:0 Z:0 H:0 PV:0 N:0 C:0));

        decl_test_case!(carry_flag_preserved,
            inputs: 0x21, (C:1);
            outputs: 0x22, (S:0 Z:0 H:0 PV:0 N:0 C:1));

        decl_test_case!(signed_flag_set,
            inputs: 0xa0, (C:0);
            outputs: 0xa1, (S:1 Z:0 H:0 PV:0 N:0 C:0));

        decl_test_case!(zero_flag_set,
            inputs: 0xff, (C:0);
            outputs: 0x00, (S:0 Z:1 H:1 PV:0 N:0 C:0));

        decl_test_case!(halfcarry_flag_set,
            inputs: 0x2f, (C:0);
            outputs: 0x30, (S:0 Z:0 H:1 PV:0 N:0 C:0));

        decl_test_case!(overflow_flag_set,
            inputs: 0x7f, (C:0);
            outputs: 0x80, (S:1 Z:0 H:1 PV:1 N:0 C:0));
    });

    decl_scenario!(alu_dec8, {
        macro_rules! decl_test_case {
            ($cname:ident, inputs: $a:expr, ($($f0:tt)+); outputs: $c:expr, ($($flags:tt)+)) => {
                decl_test!($cname, {
                    let alu = ALU::new();
                    let initial_flags = flags_apply!(0, $($f0)+);
                    let mut flags = initial_flags;
                    let c = alu.dec8_with_flags($a, &mut flags);
                    assert_result!(HEX8, "result", $c, c);
                    assert_result!(BIN8, "flags", flags_apply!(initial_flags, $($flags)+), flags);
                });
            };
        }

        decl_test_case!(base_case,
            inputs: 0x22, (C:0);
            outputs: 0x21, (S:0 Z:0 H:0 PV:0 N:1 C:0));

        decl_test_case!(carry_flag_preserved,
            inputs: 0x22, (C:1);
            outputs: 0x21, (S:0 Z:0 H:0 PV:0 N:1 C:1));

        decl_test_case!(signed_flag_set,
            inputs: 0xa1, (C:0);
            outputs: 0xa0, (S:1 Z:0 H:0 PV:0 N:1 C:0));

        decl_test_case!(zero_flag_set,
            inputs: 0x01, (C:0);
            outputs: 0x00, (S:0 Z:1 H:0 PV:0 N:1 C:0));

        decl_test_case!(halfcarry_flag_set,
            inputs: 0x30, (C:0);
            outputs: 0x2f, (S:0 Z:0 H:1 PV:0 N:1 C:0));

        decl_test_case!(overflow_flag_set,
            inputs: 0x80, (C:0);
            outputs: 0x7f, (S:0 Z:0 H:1 PV:1 N:1 C:0));
    });

    decl_scenario!(alu_add16, {
        macro_rules! decl_test_case {
            ($cname:ident, inputs: $a:expr, $b:expr, ($($f0:tt)+); outputs: $c:expr, ($($flags:tt)+)) => {
                decl_test!($cname, {
                    let alu = ALU::new();
                    let initial_flags = flags_apply!(0, $($f0)+);
                    let mut flags = initial_flags;
                    let c = alu.add16_with_flags($a, $b, &mut flags);
                    assert_result!(HEX8, "result", $c, c);
                    assert_result!(BIN8, "flags", flags_apply!(initial_flags, $($flags)+), flags);
                });
            };
        }

        decl_test_case!(base_case,
            inputs: 0x6321, 0x1142, (S:0 Z:0 PV:0);
            outputs: 0x7463, (S:0 Z:0 H:0 PV:0 N:0 C:0));

        decl_test_case!(signed_flag_preserved,
            inputs: 0x6321, 0x1142, (S:1 Z:0 PV:0);
            outputs: 0x7463, (S:1 Z:0 H:0 PV:0 N:0 C:0));

        decl_test_case!(zero_flag_preserved,
            inputs: 0x6321, 0x1142, (S:0 Z:1 PV:0);
            outputs: 0x7463, (S:0 Z:1 H:0 PV:0 N:0 C:0));

        decl_test_case!(halfcarry_flag_set,
            inputs: 0x1f21, 0x2142, (S:0 Z:0 PV:0);
            outputs: 0x4063, (S:0 Z:0 H:1 PV:0 N:0 C:0));

        decl_test_case!(overflow_flag_preserved,
            inputs: 0x6321, 0x1142, (S:0 Z:0 PV:1);
            outputs: 0x7463, (S:0 Z:0 H:0 PV:1 N:0 C:0));

        decl_test_case!(carry_flag_set,
            inputs: 0xf321, 0x2142, (S:0 Z:0 PV:0);
            outputs: 0x1463, (S:0 Z:0 H:0 PV:0 N:0 C:1));
    });

    decl_scenario!(alu_bitwise_and, {
        macro_rules! decl_test_case {
            ($cname:ident, inputs: $a:expr, $b:expr; expected_output: $c:expr; expected_flags: $($flags:tt)+) => {
                decl_test!($cname, {
                    let alu = ALU::new();
                    let mut flags = 0;
                    let c = alu.bitwise_and($a, $b, &mut flags);
                    assert_result!(BIN8, "result", $c, c);
                    assert_result!(BIN8, "flags", flags_apply!(0, $($flags)+), flags);
                });
            };
        }

        decl_test_case!(base_case,
            inputs: 0b0010_1001, 0b0110_0110;
            expected_output: 0b0010_0000;
            expected_flags: S:0 Z:0 H:1 PV:0 N:0 C:0);

        decl_test_case!(signed_flag_set,
            inputs: 0b1000_0000, 0b1111_1111;
            expected_output: 0b1000_0000;
            expected_flags: S:1 Z:0 H:1 PV:0 N:0 C:0);

        decl_test_case!(zero_flag_set,
            inputs: 0b0000_0000, 0b1111_1111;
            expected_output: 0b0000_0000;
            expected_flags: S:0 Z:1 H:1 PV:1 N:0 C:0);

        decl_test_case!(parity_flag_set,
            inputs: 0b0101_1010, 0b1111_1111;
            expected_output: 0b0101_1010;
            expected_flags: S:0 Z:0 H:1 PV:1 N:0 C:0);
    });

    decl_scenario!(alu_bitwise_xor, {
        macro_rules! decl_test_case {
            ($cname:ident, inputs: $a:expr, $b:expr; expected_output: $c:expr; expected_flags: $($flags:tt)+) => {
                decl_test!($cname, {
                    let alu = ALU::new();
                    let mut flags = 0;
                    let c = alu.bitwise_xor($a, $b, &mut flags);
                    assert_result!(BIN8, "result", $c, c);
                    assert_result!(BIN8, "flags", flags_apply!(0, $($flags)+), flags);
                });
            };
        }

        decl_test_case!(base_case,
            inputs: 0b0010_1001, 0b0110_0110;
            expected_output: 0b0100_1111;
            expected_flags: S:0 Z:0 H:0 PV:0 N:0 C:0);

        decl_test_case!(signed_flag_set,
            inputs: 0b1010_1000, 0b0110_0110;
            expected_output: 0b1100_1110;
            expected_flags: S:1 Z:0 H:0 PV:0 N:0 C:0);

        decl_test_case!(zero_flag_set,
            inputs: 0b1111_1111, 0b1111_1111;
            expected_output: 0b0000_0000;
            expected_flags: S:0 Z:1 H:0 PV:1 N:0 C:0);

        decl_test_case!(parity_flag_set,
            inputs: 0b0010_1001, 0b0111_0110;
            expected_output: 0b0101_1111;
            expected_flags: S:0 Z:0 H:0 PV:1 N:0 C:0);
    });

    decl_scenario!(alu_bitwise_or, {
        macro_rules! decl_test_case {
            ($cname:ident, inputs: $a:expr, $b:expr; expected_output: $c:expr; expected_flags: $($flags:tt)+) => {
                decl_test!($cname, {
                    let alu = ALU::new();
                    let mut flags = 0;
                    let c = alu.bitwise_or($a, $b, &mut flags);
                    assert_result!(BIN8, "result", $c, c);
                    assert_result!(BIN8, "flags", flags_apply!(0, $($flags)+), flags);
                });
            };
        }

        decl_test_case!(base_case,
            inputs: 0b0001_1001, 0b0001_0110;
            expected_output: 0b0001_1111;
            expected_flags: S:0 Z:0 H:0 PV:0 N:0 C:0);

        decl_test_case!(signed_flag_set,
            inputs: 0b0000_1001, 0b1000_0110;
            expected_output: 0b1000_1111;
            expected_flags: S:1 Z:0 H:0 PV:0 N:0 C:0);

        decl_test_case!(zero_flag_set,
            inputs: 0b0000_0000, 0b0000_0000;
            expected_output: 0b0000_0000;
            expected_flags: S:0 Z:1 H:0 PV:1 N:0 C:0);

        decl_test_case!(parity_flag_set,
            inputs: 0b0000_1001, 0b0000_0110;
            expected_output: 0b0000_1111;
            expected_flags: S:0 Z:0 H:0 PV:1 N:0 C:0);
    });

    decl_scenario!(alu_rotate_left, {
        macro_rules! decl_test_case {
            ($cname:ident, inputs: $ival:expr, $carry:expr; outputs: $oval:expr, ($($oflags:tt)+)) => {
                decl_test!($cname, {
                    let alu = ALU::new();
                    let mut flags = flags_apply!(0, C:[$carry != 0]);
                    let c = alu.rotate_left($ival, &mut flags);
                    assert_result!(BIN8, "result", $oval, c);
                    assert_result!(BIN8, "flags", flags_apply!(flags, $($oflags)+), flags);
                });
            };
        }

        decl_test_case!(carry_flag_reset,
            inputs: 0b0000_0011, 0;
            outputs: 0b0000_0110, (C:0 H:0 N:0));

        decl_test_case!(carry_flag_set,
            inputs: 0b1000_0011, 0;
            outputs: 0b0000_0110, (C:1 H:0 N:0));

        decl_test_case!(prev_carry_shifted,
            inputs: 0b0000_0011, 1;
            outputs: 0b0000_0111, (C:0 H:0 N:0));
    });

    decl_scenario!(alu_rotate_left_with_carry, {
        macro_rules! decl_test_case {
            ($cname:ident, inputs: $ival:expr; outputs: $oval:expr, ($($oflags:tt)+)) => {
                decl_test!($cname, {
                    let alu = ALU::new();
                    let mut flags = 0;
                    let c = alu.rotate_left_with_carry($ival, &mut flags);
                    assert_result!(BIN8, "result", $oval, c);
                    assert_result!(BIN8, "flags", flags_apply!(flags, $($oflags)+), flags);
                });
            };
        }

        decl_test_case!(carry_flag_reset,
            inputs: 0b0000_0011;
            outputs: 0b0000_0110, (C:0 H:0 N:0));

        decl_test_case!(carry_flag_set,
            inputs: 0b1000_0011;
            outputs: 0b0000_0111, (C:1 H:0 N:0));
    });

    decl_scenario!(alu_rotate_right, {
        macro_rules! decl_test_case {
            ($cname:ident, inputs: $ival:expr, $carry:expr; outputs: $oval:expr, ($($oflags:tt)+)) => {
                decl_test!($cname, {
                    let alu = ALU::new();
                    let mut flags = flags_apply!(0, C:[$carry != 0]);
                    let c = alu.rotate_right($ival, &mut flags);
                    assert_result!(BIN8, "result", $oval, c);
                    assert_result!(BIN8, "flags", flags_apply!(flags, $($oflags)+), flags);
                });
            };
        }

        decl_test_case!(carry_flag_reset,
            inputs: 0b0000_1100, 0;
            outputs: 0b0000_0110, (C:0 H:0 N:0));

        decl_test_case!(carry_flag_set,
            inputs: 0b0000_1101, 0;
            outputs: 0b0000_0110, (C:1 H:0 N:0));

        decl_test_case!(prev_carry_shifted,
            inputs: 0b0000_1100, 1;
            outputs: 0b1000_0110, (C:0 H:0 N:0));
    });

    decl_scenario!(alu_rotate_right_with_carry, {
        macro_rules! decl_test_case {
            ($cname:ident, inputs: $ival:expr; outputs: $oval:expr, ($($oflags:tt)+)) => {
                decl_test!($cname, {
                    let alu = ALU::new();
                    let mut flags = 0;
                    let c = alu.rotate_right_with_carry($ival, &mut flags);
                    assert_result!(BIN8, "result", $oval, c);
                    assert_result!(BIN8, "flags", flags_apply!(flags, $($oflags)+), flags);
                });
            };
        }

        decl_test_case!(carry_flag_reset,
            inputs: 0b0000_1100;
            outputs: 0b0000_0110, (C:0 H:0 N:0));

        decl_test_case!(carry_flag_set,
            inputs: 0b0000_1101;
            outputs: 0b1000_0110, (C:1 H:0 N:0));
    });

    decl_scenario!(alu_bcd_adjust, {
        macro_rules! decl_test_case {
            ($cname:ident, inputs: $ival:expr, ($($iflags:tt)+); outputs: $oval:expr, ($($oflags:tt)+)) => {
                decl_test!($cname, {
                    let alu = ALU::new();
                    let mut flags = flags_apply!(0, $($iflags)+);
                    let c = alu.bcd_adjust($ival, &mut flags);
                    assert_result!(HEX8, "result", $oval, c);
                    assert_result!(BIN8, "flags", flags_apply!(flags, $($oflags)+), flags);
                });
            };
            ($cname:ident, $input:expr, ($($flags:tt)+), $expected:expr) => {
                decl_test!($cname, {
                    let alu = ALU::new();
                    let mut flags = flags_apply!(0, $($flags)+);
                    let c = alu.bcd_adjust($input, &mut flags);
                    assert_result!(HEX8, "result", $expected, c);
                });
            };
        }

        decl_scenario!(after_addition, {
            decl_test_case!(low_nibble_adjusted,
                inputs: 0x04, (N:0 C:0 H:0);
                outputs: 0x04, (N:0 C:0 H:0 Z:0 S:0));

            decl_test_case!(low_nibble_with_overflow,
                inputs: 0x0b, (N:0 C:0 H:0);
                outputs: 0x11, (N:0 C:0 H:1 Z:0 S:0));

            decl_test_case!(low_nibble_with_carry,
                inputs: 0x02, (N:0 C:0 H:1);
                outputs: 0x08, (N:0 C:0 H:0 Z:0 S:0));

            decl_test_case!(high_nibble_adjusted,
                inputs: 0x40, (N:0 C:0 H:0);
                outputs: 0x40, (N:0 C:0 H:0 Z:0 S:0));

            decl_test_case!(high_nibble_with_overflow,
                inputs: 0xb0, (N:0 C:0 H:0);
                outputs: 0x10, (N:0 C:1 H:0 Z:0 S:0));

            decl_test_case!(high_nibble_with_carry,
                inputs: 0x20, (N:0 C:1 H:0);
                outputs: 0x80, (N:0 C:1 H:0 Z:0 S:1));
        });

        decl_scenario!(after_subtraction, {
            decl_test_case!(low_nibble_adjusted,
                inputs: 0x04, (N:1 C:0 H:0);
                outputs: 0x04, (N:1 C:0 H:0 Z:0 S:0));

            decl_test_case!(low_nibble_with_overflow,
                inputs: 0x0b, (N:1 C:0 H:0);
                outputs: 0x05, (N:1 C:0 H:0 Z:0 S:0));

            decl_test_case!(low_nibble_with_carry,
                inputs: 0x08, (N:1 C:0 H:1);
                outputs: 0x02, (N:1 C:0 H:0 Z:0 S:0));

            decl_test_case!(high_nibble_adjusted,
                inputs: 0x40, (N:1 C:0 H:0);
                outputs: 0x40, (N:1 C:0 H:0 Z:0 S:0));

            decl_test_case!(high_nibble_with_overflow,
                inputs: 0xb0, (N:1 C:0 H:0);
                outputs: 0x50, (N:1 C:1 H:0 Z:0 S:0));

            decl_test_case!(high_nibble_with_carry,
                inputs: 0x80, (N:1 C:1 H:0);
                outputs: 0x20, (N:1 C:1 H:0 Z:0 S:0));
        });
    });
}
