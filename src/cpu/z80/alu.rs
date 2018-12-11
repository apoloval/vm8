
#[derive(Clone, Copy, Default)]
struct PreFlags {
    pub add: u8,
    pub adc: u8,
    pub sub: u8,
    pub sbc: u8,
}

// A table with precalculated flags, indexed by [(oldval << 8) & newval]
type PreFlagsTable = Vec<PreFlags>;

pub struct ALU {
    pre_flags: PreFlagsTable,
}

impl ALU {
    pub fn new() -> Self {
        Self { pre_flags: Self::init_pre_flags() }
    }

    #[inline]
    pub fn add8(&self, a: u8, b: u8) -> u8 {
        ((a as u16) + (b as u16)) as u8
    }

    #[inline]
    pub fn adc8(&self, a: u8, b: u8, c: u8) -> u8 {
        if c > 0 {
            ((a as u16) + (b as u16) + 1) as u8
        } else {
            self.add8(a, b)
        }
    }

    #[inline]
    pub fn add8_with_flags(&self, a: u8, b: u8, flags: &mut u8) -> u8 {
        let c = self.add8(a, b);
        let index = Self::pre_flags_index(a, c);
        *flags = self.pre_flags[index].add;
        c
    }

    #[inline]
    pub fn adc8_with_flags(&self, a: u8, b: u8, flags: &mut u8) -> u8 {
        let c = self.adc8(a, b, flag!(C, *flags));
        let index = Self::pre_flags_index(a, c);
        *flags = self.pre_flags[index].add;
        c
    }

    #[inline]
    pub fn inc8_with_flags(&self, a: u8, flags: &mut u8) -> u8 {
        let c = self.add8(a, 1);
        let index = Self::pre_flags_index(a, c);
        *flags = (*flags & 0x01) | (self.pre_flags[index].add & 0xfe);
        c
    }

    #[inline]
    pub fn add16(&self, a: u16, b: u16) -> (u16, bool) {
        let c = (a as u32) + (b as u32);
        (c as u16, c > 0x00ff)
    }

    #[inline]
    pub fn add16_with_flags(&self, a: u16, b: u16, flags: &mut u8) -> u16 {
        let (c, carry) = self.add16(a, b);
        *flags = flags_apply!(*flags,
            H:[(a & 0x0fff) + (b & 0x0fff) > 0x1000]
            N:0
            C:[carry]
        );
        c
    }

    #[inline]
    pub fn sub8(&self, a: u8, b: u8) -> (u8, bool) {
        let c = (a as i16) - (b as i16);
        (c as u8, c > 0xff)
    }

    #[inline]
    pub fn sub8_with_flags(&self, a: u8, b: u8, flags: &mut u8) -> u8 {
        let (c, _) = self.sub8(a, b);
        let index = Self::pre_flags_index(a, c);
        *flags = self.pre_flags[index].sub;
        c
    }

    #[inline]
    pub fn dec8_with_flags(&self, a: u8, flags: &mut u8) -> u8 {
        let (c, _) = self.sub8(a, 1);
        let index = Self::pre_flags_index(a, c);
        *flags = (*flags & 0x01) | (self.pre_flags[index].sub & 0xfe);
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

    fn init_pre_flags() -> PreFlagsTable {
        let mut pre_flags = vec![PreFlags::default(); 256*256];
        for a in 0..=255 {
            for c in 0..=255 {
                let index = Self::pre_flags_index(a, c);
                {
                    let b = ((c as i16) - (a as i16)) as u8;
                    pre_flags[index].add = flags_apply!(0,
                        S:[(c & 0x80) != 0]
                        Z:[c == 0]
                        H:[(a & 0x0f) > (c & 0x0f)]
                        PV:[(a ^ b ^ 0x80) & (b ^ c) & 0x80 != 0]
                        N:0
                        C:[c < a]
                    );
                }
                {
                    let b = ((a as i16) - (c as i16)) as u8;
                    pre_flags[index].sub = flags_apply!(0,
                        S:[(c & 0x80) != 0]
                        Z:[c == 0]
                        H:[(a & 0x0f) < (c & 0x0f)]
                        PV:[(a ^ b) & ((a ^ c) & 0x80) != 0]
                        N:1
                        C:0
                    );
                }
            }
        }
        pre_flags
    }

    #[inline]
    fn pre_flags_index(oldval: u8, newval: u8) -> usize {
        ((oldval as usize) << 8) | (newval as usize)
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
}