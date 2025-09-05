use std::collections::HashMap;
use std::io;
use std::path::Path;

use crate::cpu::w65c02;
use crate::cpu::w65c02::Bus as W65C02Bus;
use crate::vid::nxvid;
use crate::mem;
use crate::sys::nexus::cmd::Command;
use crate::sys::nexus::bus::Bus;

pub struct System {    
    cpu: w65c02::CPU,
    bus: Bus,
    breakpoints: HashMap<u16, ()>,
    cycles: usize,
}

impl System {
    pub fn new(bios_path: &Path) -> io::Result<Self> {
        let bios = mem::ROM::load_from_file(bios_path)?;
        let vid = nxvid::NXVID::with_window_title(
            "Nexus Computer System emulator");
        let bus = Bus::new(vid, bios);

        Ok(Self {
            cpu: w65c02::CPU::new(),
            bus,
            breakpoints: HashMap::new(),
            cycles: 0,
        })        
    }

    pub fn exec_cmd(&mut self, cmd: Command) {
        match cmd {
            Command::StatusShow => self.exec_status(),
            Command::Reset => self.exec_reset(),
            Command::Step => self.exec_step(),
            Command::Resume => self.exec_resume(),
            Command::BreakSet { addr } => self.exec_break_set(addr),
            Command::BreakShow => self.exec_break_show(),
            Command::BreakDelete { addr } => self.exec_break_delete(addr),
            Command::MemShow { addr } => self.exec_mem_show(addr),
            _ => unreachable!(),
        }
    }

    fn exec_reset(&mut self) {
        self.cpu.reset(&mut self.bus);
    }

    fn exec_break_set(&mut self, addr: u16) {
        self.breakpoints.insert(addr, ());
    }

    fn exec_break_show(&mut self) {
        if self.breakpoints.len() > 0 {
            println!("Breakpoints:");
            for addr in self.breakpoints.keys() {
                println!("  {:04X}", addr);
            }
            println!("");
        }
    }

    fn exec_break_delete(&mut self, addr: Option<u16>) {
        if let Some(addr) = addr {
            self.breakpoints.remove(&addr);
        } else {
            self.breakpoints.clear();
        }
    }

    fn exec_mem_show(&self, addr: Option<u16>) {
        let a = addr.unwrap_or(self.cpu.pc);
        for i in (0..256).step_by(16) {
            print!("  {:04X}:", a + i);
            for j in 0..16 {
                print!(" {:02X}", self.bus.mem_read(a + i + j));
            }
            println!("");
        }
        println!("");
    }

    fn exec_status(&self) {
        println!("  CPU   : A={:02X} X={:02X} Y={:02X}  SP={:02X} PC={:04X} P={:02X}", 
            self.cpu.a, self.cpu.x, self.cpu.y, self.cpu.sp, self.cpu.pc, self.cpu.status,
        );
        println!("  Banks : {:02X} {:02X} {:02X} {:02X}", 
            self.bus.bank_reg(0), self.bus.bank_reg(1), self.bus.bank_reg(2), self.bus.bank_reg(3),
        );
        println!("");
    }

    fn exec_step(&mut self) {
        let pc = self.cpu.pc;
        let inst = self.cpu.exec(&mut self.bus);
        print!("{:04X}:   ", pc);
        for i in 0..3 {
            if i < inst.len() {
                print!("{:02X} ", self.bus.mem_read(pc.wrapping_add(i as u16)));
            } else {
                print!("   ");
            }
        }
        println!("   {}", inst);
        self.bus.refresh_all();
    }

    fn exec_resume(&mut self) {
        loop {
            let inst = self.cpu.exec(&mut self.bus);
            self.cycles += inst.cycles;
            if self.breakpoints.contains_key(&self.cpu.pc) {
                println!("Breakpoint at {:04X}", self.cpu.pc);
                break;
            }

            // TODO: adjust this to the clock speed, etc.
            if self.cycles > 120_000 {
                self.bus.refresh_all();
                self.cycles = 0;
            }
        }
        self.bus.refresh_all();
    }
}
