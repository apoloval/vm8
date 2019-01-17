/// A table with precalculated flags
struct FlagsTable {
    add: Vec<u8>,
    sub: Vec<u8>,
    and: Vec<u8>,
}

impl FlagsTable {
    /// Returns a flags table with pre-computed values.
    pub fn new() -> Self {
        let mut add_flags = vec![0u8; 256*256];
        let mut sub_flags = vec![0u8; 256*256];
        let mut and_flags = vec![0u8; 256];
        for a in 0u8..=255 {
            and_flags[a as usize] = Self::flags_for_bitwise(a, true);
            for c in 0u8..=255 {
                let index = Self::index_of_binops(a, c);
                add_flags[index] = Self::flags_for_add(a, c);
                sub_flags[index] = Self::flags_for_sub(a, c);
            }
        }
        Self {
            add: add_flags,
            sub: sub_flags,
            and: and_flags,
        }
    }

    /// Returns the precomputed flags for adding
    pub fn add_flags(&self, oldval: u8, newval: u8) -> u8 {
        self.add[Self::index_of_binops(oldval, newval)]
    }

    /// Returns the precomputed flags for subtracting
    pub fn sub_flags(&self, oldval: u8, newval: u8) -> u8 {
        self.sub[Self::index_of_binops(oldval, newval)]
    }

    /// Returns the precomputed flags for bitwise AND
    pub fn and_flags(&self, newval: u8) -> u8 {
        self.and[newval as usize]
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

    fn signed_flag(c: u8) -> bool { (c & 0x80) != 0 }
    fn zero_flag(c: u8) -> bool { c == 0 }
    fn halfcarry_flag(a: u8, c: u8) -> bool { (a & 0x0f) > (c & 0x0f) }
    fn halfborrow_flag(a: u8, c: u8) -> bool { (a & 0x0f) < (c & 0x0f) }
    fn overflow_flag(a: u8, b: u8, c: u8) -> bool { (a ^ b ^ 0x80) & (b ^ c) & 0x80 != 0 }
    fn underflow_flag(a: u8, b: u8, c: u8) -> bool { (a ^ b) & ((a ^ c) & 0x80) != 0 }
    fn carry_flag(a: u8, c: u8) -> bool { c < a }
    fn borrow_flag(a: u8, c: u8) -> bool { c > a }

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
    pub fn add16(&self, a: u16, b: u16) -> (u16, bool) {
        let c = (a as u32) + (b as u32);
        (c as u16, c > 0x00ff)
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

    #[inline]
    pub fn rotate_left(&self, val: u8, carry: u8, flags: &mut u8) -> u8 {
        let new_carry = val >> 7;
        *flags = flags_apply!(*flags, C:[new_carry > 0] H:0 N:0);
        (val << 1) | carry
    }

    #[inline]
    pub fn rotate_right(&self, val: u8, carry: u8, flags: &mut u8) -> u8 {
        let new_carry = val & 0x01;
        *flags = flags_apply!(*flags, C:[new_carry > 0] H:0 N:0);
        (val >> 1) | (carry << 7)
    }

    #[inline]
    pub fn bitwise_and(&self, a: u8, b:u8, flags: &mut u8) -> u8 {
        let c = a & b;
        *flags = self.flags.and_flags(c);
        c
    }
}

#[cfg(test)]
mod test {
    use super::*;

    decl_test!(test_alu_add8, {
        let alu = ALU::new();
        assert_eq!(3, alu.add8(1, 2));
        assert_eq!(-1 as i8 as u8, alu.add8(1, -2 as i8 as u8));
        assert_eq!(-1 as i8 as u8, alu.add8(1, -2 as i8 as u8));
    });

    decl_suite!(test_alu_add16, {
        decl_test!(no_carry, {
            let alu = ALU::new();
            let (c, carry) =  alu.add16(0x20, 0x42);
            assert_eq!(0x62, c);
            assert!(!carry);
        });
        decl_test!(carry, {
            let alu = ALU::new();
            let (c, carry) =  alu.add16(0xe012, 0x4245);
            assert_eq!(0x2257, c);
            assert!(carry);
        });
    });

    decl_test!(test_alu_bitwise_and, {
        let alu = ALU::new();
        let mut flags = 0;
        let c = alu.bitwise_and(0b0010_1001, 0b0110_0110, &mut flags);
        assert_result!(BIN8, "result", 0b0010_0000, c);
        assert_result!(BIN8, "flags", flags_apply!(0, S:0 Z:0 H:1 PV:0 N:0 C:0), flags);
    });

    decl_test!(test_alu_bitwise_and_when_signed, {
        let alu = ALU::new();
        let mut flags = 0;
        let c = alu.bitwise_and(0b1000_0000, 0b1111_1111, &mut flags);
        assert_result!(BIN8, "result", 0b1000_0000, c);
        assert_result!(BIN8, "flags", flags_apply!(0, S:1 Z:0 H:1 PV:0 N:0 C:0), flags);
    });

    decl_test!(test_alu_bitwise_and_when_zero, {
        let alu = ALU::new();
        let mut flags = 0;
        let c = alu.bitwise_and(0b0000_0000, 0b1111_1111, &mut flags);
        assert_result!(BIN8, "result", 0b0000_0000, c);
        assert_result!(BIN8, "flags", flags_apply!(0, S:0 Z:1 H:1 PV:1 N:0 C:0), flags);
    });

    decl_test!(test_alu_bitwise_and_when_parity, {
        let alu = ALU::new();
        let mut flags = 0;
        let c = alu.bitwise_and(0b0101_1010, 0b1111_1111, &mut flags);
        assert_result!(BIN8, "result", 0b0101_1010, c);
        assert_result!(BIN8, "flags", flags_apply!(0, S:0 Z:0 H:1 PV:1 N:0 C:0), flags);
    });
}