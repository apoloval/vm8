use super::bus::Bus;

const VECTOR_EMULATION_RESET: u16 = 0xFFFC;
const VECTOR_EMULATION_IRQBRK: u16 = 0xFFFE;
const VECTOR_NATIVE_BRK: u16 = 0xFFE6;

// The mode the CPU is operating in. One of Emulation or Native.
pub enum Mode { Emulation, Native }

impl Default for Mode { 
    fn default() -> Self { Mode::Emulation } 
}

#[derive(Default)]
pub struct CPU {
    mode: Mode,
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
        self.mode = Mode::Emulation;
        self.pc = bus.read_word(0, VECTOR_EMULATION_RESET);
        Self::write_msb(&mut self.pc, 0x01);
        self.pbr = 0;
    }

    pub fn step<B: Bus>(&mut self, bus: &mut B) {
        match self.fetch_opcode(bus) {
            0x00 => self.brk(bus),
            _ => unimplemented!()
        }
    }

    fn fetch_opcode<B: Bus>(&mut self, bus: &mut B) -> u8 {
        let bank = match self.mode {
            Mode::Emulation => 0,
            Mode::Native => self.pbr
        };
        bus.read_byte(bank, self.pc)
    }

    fn brk<B: Bus>(&mut self, bus: &mut B) {
        match self.mode {
            Mode::Emulation => {
                self.push_byte(bus, self.p);
                self.push_word(bus, self.pc+2);
                self.pc = bus.read_word(0, VECTOR_EMULATION_IRQBRK);
            }
            Mode::Native => {
                self.push_byte(bus, self.p);
                self.push_byte(bus, self.pbr);
                self.push_word(bus, self.pc+2);
                self.push_byte(bus, 0x00);
                self.pc = bus.read_word(0, VECTOR_NATIVE_BRK);
                self.pbr = 0;
            }
        
        }
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
            Mode::Emulation => 0x0100 | (self.sp & 0x00FF),
            Mode::Native => self.sp
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::w65c816::bus::FakeBus;

    use rstest::*;

    #[fixture]
    fn bus() -> impl Bus {
        FakeBus::new()
    }

    #[fixture]
    fn cpu_emulation_mode() -> CPU {
        let mut cpu = CPU::new();
        cpu.mode = Mode::Emulation;
        cpu.pc = 0xA000;
        cpu.sp = 0x00FF;
        cpu
    }

    #[rstest]
    fn test_emulation_brk(cpu_emulation_mode: CPU, mut bus: impl Bus) {
        let mut cpu = cpu_emulation_mode;
        let pc = cpu.pc;
        let sp = cpu.stack_pointer();

        given_flags(&mut cpu, 0xAB);
        given_mem_word(&mut bus, 0, 0xA000, 0x0000);
        given_mem_word(&mut bus, 0, 0xFFFE, 0x1234);

        cpu.step(&mut bus);

        assert_eq!(bus.read_byte(0, sp-0), 0xAB);
        assert_eq!(bus.read_word(0, sp-2), pc+2);
        assert_eq!(cpu.pc, 0x1234);
    }

    fn given_flags(cpu: &mut CPU, flags: u8) {
        cpu.p = flags;
    }
 
    fn given_mem_word<B: Bus>(bus: &mut B, bank: u8, address: u16, value: u16) {
        bus.write_word(bank, address, value);
    }
}