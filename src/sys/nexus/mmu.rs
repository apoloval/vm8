pub struct MMU {
    enabled: bool,
    regs: [u8;16]
}

impl MMU {
    pub fn new() -> Self {
        Self {
            enabled: false,
            regs: [0xFF; 16],
        }
    }

    pub fn map_addr(&self, addr: u16) -> u32 {
        if self.enabled {
            let frame = (addr >> 12) as usize;
            let page = self.regs[frame] as u32;
            (page << 12) | (addr as u32 & 0xFFF)
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
    #[case(&[(0x0, 0x42)], 0x0ABC, 0x42ABC)]
    #[case(&[(0x5, 0x42)], 0x5ABC, 0x42ABC)]
    fn test_map_addr(#[case] writes: &[(u8, u8)], #[case] given: u16, #[case] expected: u32) {
        let mut mmu = MMU::new();
        for (port, val) in writes {
            mmu.write(*port, *val);
        }

        assert_eq!(mmu.map_addr(given), expected);
    }
}