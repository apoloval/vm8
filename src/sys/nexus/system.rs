use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crate::cpu::z81;
use crate::sys::nexus::Command;
use crate::sys::nexus::mmu::MMU;

pub struct System {    
    cpu: z81::CPU,
    bus: Bus,
}

impl System {
    pub fn new(bios_path: &Path) -> io::Result<Self> {
        let bios = Self::load_bios(bios_path)?;
        Ok(Self {
            cpu: z81::CPU::new(),
            bus: Bus::new(bios),
        })        
    }

    pub fn prompt(&self) -> String {
        format!("[{}]> ", self.mapped_addr_display(self.cpu.regs().pc()))
    }

    pub fn exec_cmd(&mut self, cmd: Command) {
        match cmd {
            Command::Status => self.exec_status(),
            Command::Step => self.exec_step(),
            Command::Reset => self.exec_reset(),
            Command::MemRead { addr } => self.exec_memread(addr.unwrap_or(self.mapped_addr(self.cpu.regs().pc()))),
            Command::MemWrite { addr, data } => self.exec_memwrite(addr, data),
            _ => unreachable!(),
        }
    }

    fn load_bios(path: &Path) -> io::Result<Vec<u8>> {
        let f = fs::File::open(path)?;
        let mut reader = io::BufReader::new(f);
        let mut buffer = Vec::new();
    
        reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn exec_reset(&mut self) {
        self.cpu.reset();
    }

    fn exec_memread(&self, addr: u32) {
        for org in (addr..addr+256).step_by(16) {
            print!("  {:04X}:", org);
            for offset in 0..16 {
                print!(" {:02X}", self.bus.mem[(org+offset) as usize]);
            }
            println!("")
        }
    }

    fn exec_memwrite(&mut self, addr: u32, data: Vec<u8>) {
        let mut ptr = addr;
        for byte in data {
            self.bus.mem[ptr as usize] = byte;
            ptr += 1;
        }
    }

    fn exec_status(&self) {
        self.print_cpu_regs();
        self.print_mmu_regs();
    }

    fn print_cpu_regs(&self) {
        let regs = self.cpu.regs();
        println!(
            "  CPU: AF={:04X}[{:04X}]   PC={}", 
            regs.af(), regs.af_(), self.mapped_addr_display(regs.pc()),
        );
        println!(
            "       BC={:04X}[{:04X}]   SP={}", 
            regs.bc(), regs.bc_(), self.mapped_addr_display(regs.sp()),
        );
        println!(
            "       DE={:04X}[{:04X}]", 
            regs.de(), regs.de_(),
        );
        println!(
            "       HL={:04X}[{:04X}]", 
            regs.hl(), regs.hl_(),
        );
        println!("");
    }

    fn print_mmu_regs(&self) {
        let mmu = &self.bus.mmu;
        for i in 0u16..8 {
            println!(
                "  {} R{}={}   PAGE.{:X}={:05X}   PAGE.{:X}={:05X}", 
                if i == 0 { "MMU:" } else { "    " },
                i,
                if mmu.is_enabled() { format!("{:02X}", mmu.read(i as u8)) } else { String::from("XX") }, 
                i*2,
                self.mapped_addr((i*2) << 12),
                i*2 + 1,
                self.mapped_addr((i*2 + 1) << 12),
            );
        }
        println!("");
    }

    fn exec_step(&mut self) {
        self.cpu.exec(&mut self.bus);
    }

    fn mapped_addr(&self, addr: u16) -> u32 { 
        self.bus.mmu.map_addr(addr)
    }

    fn mapped_addr_display(&self, addr: u16) -> String {
        format!("{:04X}:{:05X}", addr, self.mapped_addr(addr))
    }
}

struct Bus {
    mem: Vec<u8>,
    mmu: MMU,
}

impl Bus {
    fn new(bios: Vec<u8>) -> Self {
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
}

impl z81::Bus for Bus {    
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