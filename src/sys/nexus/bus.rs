use crate::cpu::z80;
use crate::sys::nexus::Addr;
use crate::sys::nexus::mmu::MMU;

pub struct Bus {
    mem: Vec<u8>,
    mmu: MMU,
}

impl Bus {
    pub fn new(bios: Vec<u8>) -> Self {
        let mut bus = Self {
            mem: vec![0; 1*1024*2014],
            mmu: MMU::new(),
        };
        let mut ptr = 0xF0000;
        for byte in bios {
            bus.mem[ptr] = byte;
            ptr += 1;
        }
        bus
    }

    pub fn mmu(&self) -> &MMU { &self.mmu }

    pub fn mem_read(&self, addr: Addr) -> u8 {
        match addr {
            Addr::Logical(a) => self.mem[self.mmu.map_addr(a) as usize],
            Addr::Physical(a) => self.mem[a as usize],
        }
    }
}

impl z80::Bus for Bus {    
    fn mem_read(&self, addr: u16) -> u8 { 
        let paddr = self.mmu.map_addr(addr);
        self.mem[paddr as usize] 
    }
    
    fn mem_write(&mut self, addr: u16, val: u8) {
        let paddr = self.mmu.map_addr(addr);
        if paddr < 0xE0000 {
            self.mem[paddr as usize] = val 
        }
    }

    fn io_read(&self, _port: u8) -> u8 { 0xFF}

    fn io_write(&mut self, port: u8, val: u8) { 
        match port {
            0x60..=0x67 => self.mmu.write(port - 0x60, val),
            _ => {},
        };
    }
}