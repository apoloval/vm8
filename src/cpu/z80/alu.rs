
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
    pub fn add8(&self, a: u8, b: u8, flags: &mut u8) -> u8 {
        let c = ((a as u16) + (b as u16)) as u8;
        let index = Self::pre_flags_index(a, c);
        *flags = self.pre_flags[index].add;
        c
    }

    #[inline]
    pub fn sub8(&self, a: u8, b: u8, flags: &mut u8) -> u8 {
        let c = ((a as i16) - (b as i16)) as u8;
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