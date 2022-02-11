pub struct MMU {
    enabled: bool,
    regs: [u8;8]
}

impl MMU {
    pub fn new() -> Self {
        Self {
            enabled: false,
            regs: [0xFF; 8],
        }
    }

    pub fn map_addr(&self, addr: u16) -> u32 {
        if self.enabled {
            let seg_frame = self.regs[Self::segment_of(addr)*2] as u32 & 0x0F;
            let page_offset = match Self::page_of(addr) {
                0x00 => 0x00,
                0x01 => self.regs[0] >> 4,
                0x02 => self.regs[1] & 0x0F,
                0x03 => self.regs[1] >> 4,
                0x04 => 0x00,
                0x05 => self.regs[2] >> 4,
                0x06 => self.regs[3] & 0x0F,
                0x07 => self.regs[3] >> 4,
                0x08 => 0x00,
                0x09 => self.regs[4] >> 4,
                0x0A => self.regs[5] & 0x0F,
                0x0B => self.regs[5] >> 4,
                0x0C => 0x00,
                0x0D => self.regs[6] >> 4,
                0x0E => self.regs[7] & 0x0F,
                0x0F => self.regs[7] >> 4,
                _ => unreachable!(),
            } as u32;
            (seg_frame << 16) | (page_offset << 12) | (addr as u32 & 0x0FFF)            
        } else {
            0xF0000 | (addr as u32)
        }
        
    }

    pub fn is_enabled(&self) -> bool { self.enabled }

    pub fn read(&self, reg: u8) -> u8 {
        self.regs[reg as usize & 0x07] 
    }

    pub fn write(&mut self, reg: u8, val: u8) {
        self.regs[reg as usize & 0x07] = val;
        self.enabled = true;
    }

    #[inline] fn segment_of(addr: u16) -> usize { (addr as usize) >> 14 }
    #[inline] fn page_of(addr: u16) -> usize { (addr as usize) >> 12 }
}

#[cfg(test)]
mod test {
    use rstest::*;

    use super::*;    

    #[rstest]
    #[case(&[], 0x0000, 0xF0000)]
    #[case(&[], 0x00FF, 0xF00FF)]
    #[case(&[], 0xFFFF, 0xFFFFF)]
    #[case(&[(0x0, 0x00)], 0x0ABC, 0x00ABC)]
    #[case(&[(0x0, 0x00)], 0x1ABC, 0x00ABC)]
    #[case(&[(0x0, 0x10)], 0x1ABC, 0x01ABC)]
    #[case(&[(0x0, 0x1A)], 0x1ABC, 0xA1ABC)]
    #[case(&[(0x0, 0x00), (0x1, 0x48)], 0x2ABC, 0x08ABC)]
    #[case(&[(0x0, 0x00), (0x1, 0x48)], 0x3ABC, 0x04ABC)]
    fn test_map_addr(#[case] writes: &[(u8, u8)], #[case] given: u16, #[case] expected: u32) {
        let mut mmu = MMU::new();
        for (port, val) in writes {
            mmu.write(*port, *val);
        }

        assert_eq!(mmu.map_addr(given), expected);
    }
}