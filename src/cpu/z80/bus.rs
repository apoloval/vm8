use byteorder::{ByteOrder, LittleEndian};

pub trait Bus {
    fn mem_read(&self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, val: u8);
    fn io_read(&self, addr: u8) -> u8;
    fn io_write(&mut self, addr: u8, val: u8);

    fn mem_read_word(&self, addr: u16) -> u16 {
        let data = [self.mem_read(addr), self.mem_read(addr + 1)];
        LittleEndian::read_u16(&data)        
    }

    fn mem_write_word(&mut self, addr: u16, val: u16) {
        let mut data = [0; 2];
        LittleEndian::write_u16(&mut data, val);
        self.mem_write(addr, data[0]);
        self.mem_write(addr + 1, data[1]);
    }
}

pub struct FakeBus {
    mem: [u8; 64*1024],
    io: [u8; 256],
}

impl FakeBus {
    pub fn new() -> Self {
        Self {
            mem: [0; 64*1024],
            io: [0; 256],
        }
    }
}

impl Bus for FakeBus {
    fn mem_read(&self, addr: u16) -> u8 { self.mem[addr as usize] }
    fn mem_write(&mut self, addr: u16, val: u8) { self.mem[addr as usize] = val }
    fn io_read(&self, addr: u8) -> u8 { self.io[addr as usize] }
    fn io_write(&mut self, addr: u8, val: u8) { self.io[addr as usize] = val }
}
