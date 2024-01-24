use crate::cpu::z80;
use crate::mem;
use crate::sys::nexus::Addr;
use crate::sys::nexus::mmu::MMU;
use crate::vid::nxgfx;

pub trait Device {
    fn mem_read(&self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, val: u8);
    fn io_read(&self, port: u8) -> u8;
    fn io_write(&mut self, port: u8, val: u8);
    fn refresh(&mut self);
}

impl Device for nxgfx::NXGFX216 {
    fn mem_read(&self, _addr: u16) -> u8 { 0 }
    fn mem_write(&mut self, addr: u16, val: u8) {
        self.vram_write(addr, val)
    }
    fn io_read(&self, _port: u8) -> u8 { 0 }
    fn io_write(&mut self, port: u8, val: u8) {
        self.io_write(port, val)
    }
    fn refresh(&mut self) {
        self.refresh_screen();
    }
}

impl Device for mem::ROM {
    fn mem_read(&self, addr: u16) -> u8 { self.read_byte(addr as usize) }
    fn mem_write(&mut self, _addr: u16, _val: u8) {}
    fn io_read(&self, _port: u8) -> u8 { 0 }
    fn io_write(&mut self, _port: u8, _val: u8) {}
    fn refresh(&mut self) {}
}

enum Segment {
    RAM,
    MMU,
    Device(usize),
}

impl Segment {
    fn from_mem_addr(addr: u32) -> Self {
        match addr {
            0x00000..=0x7FFFF => Self::RAM,
            0x80000..=0xFFFFF => Self::Device(((addr & 0xF0000) >> 16) as usize - 8),
            _ => panic!("Invalid address"),
        }
    }

    fn from_io_addr(addr: u8) -> Self {
        match addr {
            0x60..=0x67 => Self::MMU,
            0x80..=0xFF => Self::Device(((addr & 0xF0) >> 4) as usize),
            _ => panic!("Invalid address"),
        }
    }
}

pub struct Bus {
    mem: Vec<u8>,
    mmu: MMU,
    devs: [Option<Box<dyn Device>>; 8],
}

impl Bus {
    pub fn new() -> Self {
        let bus = Self {
            mem: vec![0; 512*1024],
            mmu: MMU::new(),
            devs: Default::default(),
        };
        bus
    }

    pub fn attach(&mut self, dev: Box<dyn Device>, idx: usize) {
        if idx > 7 {
            panic!("Invalid device index");
        }
        self.devs[idx] = Some(dev);
    }

    pub fn refresh_all(&mut self) {
        for dev in self.devs.iter_mut() {
            if let Some(dev) = dev {
                dev.refresh();
            }
        }
    }

    pub fn mmu(&self) -> &MMU { &self.mmu }

    pub fn mem_read(&self, addr: Addr) -> u8 {
        let dest = match addr {
            Addr::Logical(a) => self.mmu.map_addr(a) as u16,
            Addr::Physical(a) => a as u16,
        };
        z80::Bus::mem_read(self, dest)
    }
}

impl z80::Bus for Bus {    
    fn mem_read(&self, addr: u16) -> u8 { 
        let paddr = self.mmu.map_addr(addr);
        match Segment::from_mem_addr(paddr) {
            Segment::RAM => self.mem[paddr as usize],
            Segment::Device(idx) => {
                if let Some(dev) = &self.devs[idx] {
                    dev.mem_read(paddr as u16)
                } else {
                    0xFF
                }
            },
            _ => 0xFF,
        }
    }
    
    fn mem_write(&mut self, addr: u16, val: u8) {
        let paddr = self.mmu.map_addr(addr);
        match Segment::from_mem_addr(paddr) {
            Segment::RAM => self.mem[paddr as usize] = val,
            Segment::Device(idx) => {
                if let Some(dev) = &mut self.devs[idx] {
                    dev.mem_write(paddr as u16, val);
                }
            },
            _ => {},
        }
    }

    fn io_read(&self, _port: u8) -> u8 { 0xFF}

    fn io_write(&mut self, port: u8, val: u8) { 
        match Segment::from_io_addr(port) {
            Segment::MMU => self.mmu.write(port & 0x07, val),
            Segment::Device(idx) => {
                if let Some(dev) = &mut self.devs[idx] {
                    dev.io_write(port & 0x0F, val);
                }
            },
            _ => {},
        };
    }
}