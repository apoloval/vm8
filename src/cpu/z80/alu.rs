
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
    pub fn add8_with_flags(&self, a: u8, b: u8, flags: &mut u8) -> u8 {
        let c = self.add8(a, b);
        let index = Self::pre_flags_index(a, c);
        *flags = self.pre_flags[index].add;
        c
    }

    #[inline]
    pub fn add16(&self, a: u16, b: u16) -> u16 {
        ((a as u32) + (b as u32)) as u16
    }

    #[inline]
    pub fn sub8(&self, a: u8, b: u8) -> u8 {
        ((a as i16) - (b as i16)) as u8
    }

    #[inline]
    pub fn sub8_with_flags(&self, a: u8, b: u8, flags: &mut u8) -> u8 {
        let c = self.sub8(a, b);
        let index = Self::pre_flags_index(a, c);
        *flags = self.pre_flags[index].sub;
        c
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
                        C:[c > a]
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

    #[test]
    fn test_alu_add8() {
        let alu = ALU::new();
        assert_eq!(3, alu.add8(1, 2));
        assert_eq!(-1 as i8 as u8, alu.add8(1, -2 as i8 as u8));
        assert_eq!(-1 as i8 as u8, alu.add8(1, -2 as i8 as u8));
    }

    #[test]
    fn test_alu_add16() {
        let alu = ALU::new();
        assert_eq!(3, alu.add16(1, 2));
        assert_eq!(-1 as i16 as u16, alu.add16(1, -2 as i16 as u16));
        assert_eq!(-1 as i16 as u16, alu.add16(1, -2 as i16 as u16));
    }
}