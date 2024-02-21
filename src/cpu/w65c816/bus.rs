use super::Bus;

pub struct Fake {
    banks: Vec<Vec<u8>>
}

impl Fake {
    pub fn new() -> Self {
        Self {
            banks: vec![vec![0; 64*1024]; 256],
        }
    }
}

impl Bus for Fake {
    fn read_byte(&self, bank: u8, addr: u16) -> u8 { 
        self.banks[bank as usize][addr as usize] 
    }

    fn write_byte(&mut self, bank: u8, addr: u16, val: u8) { 
        self.banks[bank as usize][addr as usize] = val 
    }
}