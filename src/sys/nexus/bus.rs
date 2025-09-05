use crate::cpu::w65c02;
use crate::mem;
use crate::vid::nxvid;

pub trait Device {
    fn mem_read(&self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, val: u8);
    fn io_read(&self, port: u8) -> u8;
    fn io_write(&mut self, port: u8, val: u8);
    fn refresh(&mut self);
}

enum Target {
    RAM { offset: u16 },
    Banked { dev: u8, offset: u16 },
    IO { dev: u8, port: u8 },
    BIOS { offset: u16 },
}

pub struct Bus {
    ram: Vec<u8>,
    bios: mem::ROM,
    bank_regs: [u8; 4],
    vid: nxvid::NXVID,
    devs: [Option<Box<dyn Device>>; 16],
}

impl Bus {
    pub fn new(vid: nxvid::NXVID, bios: mem::ROM) -> Self {
        let bus = Self {
            ram: vec![0; 32*1024],
            bios,
            bank_regs: [0xFF; 4],
            vid,
            devs: Default::default(),
        };
        bus
    }

    pub fn bank_reg(&self, i: usize) -> u8 {
        if i > 3 {
            panic!("Invalid bank register index");
        }
        self.bank_regs[i]
    }

    #[allow(dead_code)]
    pub fn attach(&mut self, dev: Box<dyn Device>, idx: usize) {
        if idx < 2 || idx > 16 {
            panic!("Invalid device index");
        }
        self.devs[idx] = Some(dev);
    }

    pub fn refresh_all(&mut self) {
        self.vid.refresh_screen();
        for dev in self.devs.iter_mut() {
            if let Some(dev) = dev {
                dev.refresh();
            }
        }
    }

    fn target_from_addr(&self, addr: u16) -> Target {
        match addr {
            0x0000..=0x7FFF => Target::RAM { offset: addr & 0x7FFF },
            0x8000..=0xBFFF => { 
                let mmu_reg = ((addr & 0x3000) >> 12) as usize;
                let bank = self.bank_regs[mmu_reg];
                let dev = (bank & 0xF0) >> 4;
                let offset = ((bank as u16) << 12) | (addr & 0x0FFF);
                Target::Banked { dev, offset }
            }
            0xC000..=0xCFFF => Target::IO { 
                dev: ((addr & 0x0F00) >> 8) as u8, 
                port: (addr & 0xFF) as u8 ,
            },
            0xD000..=0xFFFF => Target::BIOS { offset: addr - 0xD000 },
        }
    }

    fn sys_io_write(&mut self, port: u8, val: u8) {
        match port {
            0x08 => self.bank_regs[0] = val,
            0x09 => self.bank_regs[1] = val,
            0x0A => self.bank_regs[2] = val,
            0x0B => self.bank_regs[3] = val,
            _ => (),
        }
    }
}

impl w65c02::Bus for Bus {    
    fn mem_read(&self, addr: u16) -> u8 { 
        match self.target_from_addr(addr) {
            Target::RAM { offset } => self.ram[offset as usize],
            Target::Banked { dev: 0, offset: _offset } => {
                // TODO: system bank read
                0xFF
            }
            Target::Banked { dev: 1, offset: _offset } => {
                // TODO: NXVID bank read
                0xFF
            }
            Target::Banked { dev, offset } => {
                if let Some(dev) = &self.devs[dev as usize] {
                    dev.mem_read(offset)
                } else {
                    0xFF
                }
            }
            Target::IO { dev: 0, port: _port } => {
                // TODO: system IO read
                0xFF
            }
            Target::IO { dev: 1, port: _port } => {
                // TODO: NXVID IO read
                0xFF
            }
            Target::IO { dev, port } => {
                if let Some(dev) = &self.devs[dev as usize] {
                    dev.io_read(port)
                } else {
                    0xFF
                }
            }
            Target::BIOS { offset } => self.bios.read_byte(offset as usize),
        }
    }
    
    fn mem_write(&mut self, addr: u16, val: u8) {
        match self.target_from_addr(addr) {
            Target::RAM { offset } => self.ram[offset as usize] = val,
            Target::Banked { dev: 0, offset: _offset } => {
                // TODO: system bank write
            }
            Target::Banked { dev: 1, offset: _offset } => {
                // TODO: NXVID bank write
            }
            Target::Banked { dev, offset } => {
                if let Some(dev) = &mut self.devs[dev as usize] {
                    dev.mem_write(offset, val);
                }
            }
            Target::IO { dev: 0, port } => self.sys_io_write(port, val),
            Target::IO { dev: 1, port: _port } => {
                // TODO: NXVID IO write
            }
            Target::IO { dev, port } => {
                if let Some(dev) = &mut self.devs[dev as usize] {
                    dev.io_write(port, val);
                }
            }
            _ => {},
        }
    }    
}
