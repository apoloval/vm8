pub trait Bus {
    fn mem_read(&self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, val: u8);

    fn mem_read_word(&self, addr: u16) -> u16 {
        let lo = self.mem_read(addr);
        let hi = self.mem_read(u16::wrapping_add(addr, 1));
        (hi as u16) << 8 | lo as u16
    }

    fn mem_read_word_page_wrap(&self, addr: u16) -> u16 {
        let lo = self.mem_read(addr);
        let hi = self.mem_read((addr & 0xFF00) | (u16::wrapping_add(addr, 1) & 0xFF));
        (hi as u16) << 8 | lo as u16
    }

    fn mem_write_word(&mut self, addr: u16, val: u16) {
        self.mem_write(addr, val as u8);
        self.mem_write(u16::wrapping_add(addr, 1), (val >> 8) as u8);
    }

    fn mem_write_word_page_wrap(&mut self, addr: u16, val: u16) {
        self.mem_write(addr, val as u8);
        self.mem_write((addr & 0xFF00) | (u16::wrapping_add(addr, 1) & 0xFF), (val >> 8) as u8);
    }
}

pub struct FakeBus {
    mem: [u8; 64*1024],
}

impl FakeBus {
    pub fn new() -> Self {
        Self {
            mem: [0; 64*1024],
        }
    }
}

impl Bus for FakeBus { 
    fn mem_read(&self, addr: u16) -> u8 { self.mem[addr as usize] }
    fn mem_write(&mut self, addr: u16, val: u8) { self.mem[addr as usize] = val }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_u16_page_wrap() {
        let mut bus = FakeBus::new();
        bus.mem_write(0x0000, 0x12);
        bus.mem_write(0x00FF, 0x34);
        assert_eq!(bus.mem_read_word_page_wrap(0x00FF), 0x1234);
    }

    #[test]
    fn test_write_u16_page_wrap() {
        let mut bus = FakeBus::new();
        bus.mem_write_word_page_wrap(0x00FF, 0x1234);
        assert_eq!(bus.mem_read(0x0000), 0x12);
        assert_eq!(bus.mem_read(0x00FF), 0x34);
    }
}
