use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crate::cpu::z80;
use crate::sys::nexus::Addr;
use crate::sys::nexus::cmd::Command;
use crate::sys::nexus::mmu::MMU;

pub struct System {    
    cpu: z80::CPU,
    bus: Bus,
    break_log: HashMap<u16, ()>,
    break_phy: HashMap<u32, ()>,
}

impl System {
    pub fn new(bios_path: &Path) -> io::Result<Self> {
        let bios = Self::load_bios(bios_path)?;
        Ok(Self {
            cpu: z80::CPU::new(),
            bus: Bus::new(bios),
            break_log: HashMap::new(),
            break_phy: HashMap::new(),
        })        
    }

    pub fn prompt(&self) -> String {
        format!("[{}]> ", self.display_addr(Addr::Logical(self.cpu.regs().pc())))
    }

    pub fn exec_cmd(&mut self, cmd: Command) {
        match cmd {
            Command::Status => self.exec_status(),
            Command::Reset => self.exec_reset(),
            Command::Step => self.exec_step(),
            Command::Resume => self.exec_resume(),
            Command::BreakSet { addr } => self.exec_break_set(addr),
            Command::BreakShow => self.exec_break_show(),
            Command::BreakDelete { addr } => self.exec_break_delete(addr),
            Command::MemRead { addr } => self.exec_memread(addr),
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

    fn exec_break_set(&mut self, addr: Addr) {
        match addr {
            Addr::Logical(a) => self.break_log.insert(a, ()),
            Addr::Physical(a) => self.break_phy.insert(a, ()),
        };        
    }

    fn exec_break_show(&mut self) {
        if self.break_log.len() > 0 {
            println!("Logical address:");
            for addr in self.break_log.keys() {
                println!("  {}", self.display_addr(Addr::Logical(*addr)));
            }
            println!("");
        }
        if self.break_phy.len() > 0 {
            println!("Logical address:");
            for addr in self.break_phy.keys() {
                println!("  {}", self.display_addr(Addr::Physical(*addr)));
            }
            println!("");
        }
    }

    fn exec_break_delete(&mut self, addr: Option<Addr>) {
        match addr {
            Some(Addr::Logical(it)) => { self.break_log.remove(&it); },
            Some(Addr::Physical(it)) => { self.break_phy.remove(&it); },
            None => {
                self.break_log.clear();
                self.break_phy.clear();
            },
        };            
    }

    fn exec_memread(&self, addr: Option<Addr>) {
        let addr_phy = self.resolve_addr(addr.unwrap_or(Addr::Logical(self.cpu.regs().pc())));
        for org in (addr_phy..addr_phy+256).step_by(16) {
            print!("  {:04X}:", org);
            for offset in 0..16 {
                print!(" {:02X}", self.bus.mem[(org+offset) as usize]);
            }
            println!("")
        }
    }

    fn exec_memwrite(&mut self, addr: Addr, data: Vec<u8>) {
        let addr_phy = self.resolve_addr(addr);
        let mut ptr = addr_phy;
        for byte in data {
            self.bus.mem[ptr as usize] = byte;
            ptr = ptr.wrapping_add(1);
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
            regs.af(), regs.af_(), self.display_addr(Addr::Logical(regs.pc())),
        );
        println!(
            "       BC={:04X}[{:04X}]   SP={}", 
            regs.bc(), regs.bc_(), self.display_addr(Addr::Logical(regs.sp())),
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
                self.resolve_addr(Addr::Logical((i*2) << 12)),
                i*2 + 1,
                self.resolve_addr(Addr::Logical((i*2 + 1) << 12)),
            );
        }
        println!("");
    }

    fn exec_step(&mut self) {
        self.cpu.exec(&mut self.bus);
    }

    fn exec_resume(&mut self) {
        loop {
            self.cpu.exec(&mut self.bus);
            let pc_log = self.cpu.regs().pc();
            if self.break_log.contains_key(&pc_log) {
                println!("Breakpoint at {}", self.display_addr(Addr::Logical(pc_log)));
                break;
            }
            let pc_phy = self.bus.mmu.map_addr(self.cpu.regs().pc());
            if self.break_phy.contains_key(&pc_phy) {
                println!("Breakpoint at {}", self.display_addr(Addr::Physical(pc_phy)));
                break;
            }
        }
    }

    fn resolve_addr(&self, addr: Addr) -> u32 { 
        match addr {
            Addr::Logical(a) => self.bus.mmu.map_addr(a),
            Addr::Physical(a) => a,
        }
    }

    fn display_addr(&self, addr: Addr) -> String {
        match addr {
            Addr::Logical(a) => format!("{:04X}:{:05X}", a, self.bus.mmu.map_addr(a)),
            Addr::Physical(a) => format!(":{:05X}", a),
        }
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