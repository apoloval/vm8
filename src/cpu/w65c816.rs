mod bus;
mod exec;
mod int;
mod status;

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

#[derive(Default)]
pub struct CPU {
    mode: exec::Mode,
    pc: u16,
    sp: u16,
    p: u8,
    pbr: u8,
}

impl CPU {
    pub fn new() -> CPU {
        Self::default()
    }

    pub fn reset<B: Bus>(&mut self, bus: &mut B) {
        self.mode = exec::Mode::Emulation;
        self.pc = bus.read_word(0, int::VECTOR_EMULATION_RESET);
        Self::write_msb(&mut self.pc, 0x01);
        self.pbr = 0;
    }

    pub fn step<B: Bus>(&mut self, bus: &mut B) {
        match self.fetch_opcode(bus) {
            0x00 => self.mode.brk(bus, self),
            _ => unimplemented!()
        }
    }

    fn fetch_opcode<B: Bus>(&mut self, bus: &mut B) -> u8 {
        let bank = match self.mode {
            exec::Mode::Emulation => 0,
            exec::Mode::Native => self.pbr
        };
        bus.read_byte(bank, self.pc)
    }

    fn write_msb(value: &mut u16, msb: u8) {
        *value = (*value & 0x00FF) | ((msb as u16) << 8);
    }

    fn push_byte<B: Bus>(&mut self, bus: &mut B, value: u8) {
        bus.write_byte(0, self.stack_pointer(), value);
        self.sp -= 1;
    }

    fn push_word<B: Bus>(&mut self, bus: &mut B, value: u16) {
        self.push_byte(bus, (value >> 8) as u8);
        self.push_byte(bus, value as u8);
    }

    fn stack_pointer(&self) -> u16 {
        match self.mode {
            exec::Mode::Emulation => 0x0100 | (self.sp & 0x00FF),
            exec::Mode::Native => self.sp
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::w65c816::bus::Fake;

    use rstest::*;
    
    #[rstest]
    fn test_emulation_brk(cpu_emulation_mode: CPU, mut bus: impl Bus) {
        let mut cpu = cpu_emulation_mode;
        let pc = cpu.pc;
        let sp = cpu.stack_pointer();

        given_flags(&mut cpu, 0b1010_1010);
        given_mem_word(&mut bus, 0, 0xA000, 0x0000);
        given_mem_word(&mut bus, 0, 0xFFFE, 0x1234);

        cpu.step(&mut bus);

        assert_eq!(bus.read_byte(0, sp-0), 0b1011_1010);
        assert_eq!(bus.read_word(0, sp-2), pc+2);
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.p, 0b1011_1010);
    }

    #[rstest]
    fn test_native_brk(cpu_native_mode: CPU, mut bus: impl Bus) {
        let mut cpu = cpu_native_mode;
        let pc = cpu.pc;
        let sp = cpu.stack_pointer();

        given_flags(&mut cpu, 0b1010_1010);
        given_mem_word(&mut bus, 0xB0, 0xA000, 0x0000);
        given_mem_word(&mut bus, 0, 0xFFE6, 0x1234);

        cpu.step(&mut bus);

        assert_eq!(bus.read_byte(0, sp-0), 0b1010_1010);
        assert_eq!(bus.read_byte(0, sp-1), 0xB0);
        assert_eq!(bus.read_word(0, sp-3), pc+2);
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.pbr, 0);
        assert_eq!(cpu.p, 0b1010_1010);
    }

    #[fixture]
    fn bus() -> impl Bus {
        Fake::new()
    }

    #[fixture]
    fn cpu_emulation_mode() -> CPU {
        let mut cpu = CPU::new();
        cpu.mode = exec::Mode::Emulation;
        cpu.pc = 0xA000;
        cpu.sp = 0x00FF;
        cpu
    }

    #[fixture]
    fn cpu_native_mode() -> CPU {
        let mut cpu = CPU::new();
        cpu.mode = exec::Mode::Native;
        cpu.pc = 0xA000;
        cpu.pbr = 0xB0;
        cpu.sp = 0xFFFF;
        cpu
    }

    fn given_flags(cpu: &mut CPU, flags: u8) {
        cpu.p = flags;
    }
 
    fn given_mem_word<B: Bus>(bus: &mut B, bank: u8, address: u16, value: u16) {
        bus.write_word(bank, address, value);
    }
}