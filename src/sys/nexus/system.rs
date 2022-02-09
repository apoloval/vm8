use crate::cpu::z81;
use crate::sys::nexus::Command;

pub struct System {    
    cpu: z81::CPU,
    bus: Bus,
}

impl System {
    pub fn new() -> Self {
        Self {
            cpu: z81::CPU::new(),
            bus: Bus::new(),
        }
    }

    pub fn prompt(&self) -> String {
        format!("[PC:{:04X}]> ", self.cpu.regs().pc())
    }

    pub fn exec_cmd(&mut self, cmd: Command) {
        match cmd {
            Command::Regs => self.exec_regs(),
            Command::Step => self.exec_step(),
            Command::MemRead { addr } => self.exec_memread(addr.unwrap_or(self.cpu.regs().pc())),
            Command::MemWrite { addr, data } => self.exec_memwrite(addr, data),
            _ => unreachable!(),
        }
    }

    fn exec_memread(&self, addr: u16) {
        for org in (addr..addr+256).step_by(16) {
            print!("  {:04X}:", org);
            for offset in 0..16 {
                print!(" {:02X}", self.bus.mem[(org+offset) as usize]);
            }
            println!("")
        }
    }

    fn exec_memwrite(&mut self, addr: u16, data: Vec<u8>) {
        let mut ptr = addr;
        for byte in data {
            self.bus.mem[ptr as usize] = byte;
            ptr += 1;
        }
    }

    fn exec_regs(&self) {
        let regs = self.cpu.regs();
        println!("  AF: {:04X} [{:04X}]", regs.af(), regs.af_());
        println!("  BC: {:04X} [{:04X}]", regs.bc(), regs.bc_());
        println!("  DE: {:04X} [{:04X}]", regs.de(), regs.de_());
        println!("  HL: {:04X} [{:04X}]", regs.hl(), regs.hl_());
        println!("  SP: {:04X}", regs.sp());
        println!("  PC: {:04X}", regs.pc());
    }

    fn exec_step(&mut self) {
        self.cpu.exec(&mut self.bus);
    }
}

struct Bus {
    mem: Box<[u8; 64*1024]>,    
}

impl Bus {
    fn new() -> Self {
        Self {
            mem: Box::new([0; 64*1024]),
        }
    }
}

impl z81::Bus for Bus {    
    fn mem_read(&self, addr: u16) -> u8 { self.mem[addr as usize] }
    fn mem_write(&mut self, addr: u16, val: u8) { self.mem[addr as usize] = val }
    fn io_read(&self, _port: u8) -> u8 { 0xFF}
    fn io_write(&mut self, _port: u8, _val: u8) { }
}