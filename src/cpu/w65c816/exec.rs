use super::{int, status, Bus, CPU};

// The mode the CPU is operating in. One of Emulation or Native.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode { Emulation, Native }

impl Default for Mode { 
    fn default() -> Self { Mode::Emulation } 
}

impl Mode {
    /************************ */
    /* Interrupt instructions */
    /************************ */
    pub fn brk<B: Bus>(self, bus: &mut B, cpu: &mut CPU) {
        match self {
            Mode::Emulation => {
                status::Flag::B.set(&mut cpu.p);
                cpu.push_byte(bus, cpu.p);
                cpu.push_word(bus, cpu.pc+2);
                cpu.pc = bus.read_word(0, int::VECTOR_EMULATION_IRQBRK);
            }
            Mode::Native => {
                cpu.push_byte(bus, cpu.p);
                cpu.push_byte(bus, cpu.pbr);
                cpu.push_word(bus, cpu.pc+2);
                cpu.pc = bus.read_word(0, int::VECTOR_NATIVE_BRK);
                cpu.pbr = 0;
            }
        }
    }
}
