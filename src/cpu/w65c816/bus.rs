pub trait Bus {
    fn read_byte(&self, bank: u8, addr: u16) -> u8;
    fn write_byte(&mut self, bank: u8, addr: u16, val: u8);

    fn read_word(&self, bank: u8, addr: u16) -> u16 {
        let lo = self.read_byte(bank, addr) as u16;
        let hi = self.read_byte(bank, addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    fn write_word(&mut self, bank: u8, addr: u16, val: u16) {
        let lo = val as u8;
        let hi = (val >> 8) as u8;
        self.write_byte(bank, addr, lo);
        self.write_byte(bank, addr.wrapping_add(1), hi);
    }
}

impl Bus for () {
    fn read_byte(&self, _: u8, _: u16) -> u8 { 0xFF }
    fn write_byte(&mut self, _: u8, _: u16, _: u8) {}
}

pub struct FakeBus {
    banks: Vec<Vec<u8>>
}

impl FakeBus {
    pub fn new() -> Self {
        Self {
            banks: vec![vec![0; 64*1024]; 8],
        }
    }
}

impl Bus for FakeBus {
    fn read_byte(&self, bank: u8, addr: u16) -> u8 { 
        self.banks[bank as usize][addr as usize] 
    }

    fn write_byte(&mut self, bank: u8, addr: u16, val: u8) { 
        self.banks[bank as usize][addr as usize] = val 
    }
}