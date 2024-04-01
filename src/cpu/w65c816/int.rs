use super::{Addr, AddrWrap, Bus, CPU};

pub struct Vector {
    pub native: u16,
    pub emulation: u16,
}

impl Vector {
    pub const BRK: Vector = Vector { native: 0xFFE6, emulation: 0xFFFE };
    pub const COP: Vector = Vector { native: 0xFFE4, emulation: 0xFFF4 };
    pub const RST: Vector = Vector { native: 0xFFFC, emulation: 0xFFFC };

    pub fn jump(&self, cpu: &mut CPU, bus: &mut impl Bus) {
        let vector = if cpu.regs.mode_is_emulated() { self.emulation } else { self.native };
        cpu.regs.pc_jump(bus.read_word(Addr::from(0, vector), AddrWrap::Long));
        cpu.regs.pbr_set(0);
    }
}