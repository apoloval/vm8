use crate::cpu::w65c02::{Bus, CPU};


pub enum EffectiveAddress {
    Accumulator,
    Memory(u16),
    None,
}

impl EffectiveAddress {
    pub fn read(&self, cpu: &mut CPU, bus: &mut impl Bus) -> u8 {
        match self {
            EffectiveAddress::Accumulator => cpu.a,
            EffectiveAddress::Memory(addr) => bus.mem_read(*addr),
            EffectiveAddress::None => 0,
        }
    }

    pub fn write(&self, cpu: &mut CPU, bus: &mut impl Bus, val: u8) {
        match self {
            EffectiveAddress::Accumulator => cpu.a = val,
            EffectiveAddress::Memory(addr) => bus.mem_write(*addr, val),
            EffectiveAddress::None => (),
        }
    }
}

pub enum Mode {
    Absolute,       // $4400
    AbsoluteX,      // $4400,X
    AbsoluteY,      // $4400,Y
    Accumulator,    // A
    Immediate,      // #$44
    Implied,        //
    Indirect,       // ($4400)
    IndirectX,      // ($44,X)
    IndirectY,      // ($44),Y
    Relative,       // 
    ZeroPage,       // $44
    ZeroPageX,      // $44,X
    ZeroPageY,      // $44,Y
}

impl Mode {
    pub fn fetch(&self, cpu: &mut CPU, bus: &mut impl Bus) -> EffectiveAddress {
        match self {
            Mode::Absolute => {
                let addr = cpu.fetch_word(bus);
                EffectiveAddress::Memory(addr)
            }
            Mode::AbsoluteX => {
                let addr = u16::wrapping_add(cpu.fetch_word(bus), cpu.x as u16);
                EffectiveAddress::Memory(addr)
            }            
            Mode::AbsoluteY => {
                let addr = u16::wrapping_add(bus.mem_read_word(cpu.pc), cpu.y as u16);
                cpu.pc += 2;
                EffectiveAddress::Memory(addr)
            }
            Mode::Accumulator => {
                EffectiveAddress::Accumulator
            }
            Mode::Immediate => {
                let addr = cpu.pc;
                cpu.pc += 1;
                EffectiveAddress::Memory(addr)
            }
            Mode::Implied => {
                EffectiveAddress::None
            }
            Mode::Indirect => {
                let ind = cpu.fetch_word(bus);
                let addr = bus.mem_read_word(ind);
                EffectiveAddress::Memory(addr)
            }
            Mode::IndirectX => {
                let base = cpu.fetch_byte(bus);
                let addr = cpu.zeropage_read_word(bus, base, cpu.x as u8);
                EffectiveAddress::Memory(addr)
            }
            Mode::IndirectY => {
                let base = cpu.fetch_byte(bus);
                let addr = cpu.zeropage_read_word(bus, base, 0);
                EffectiveAddress::Memory(u16::wrapping_add(addr, cpu.y as u16))
            }
            Mode::Relative => {
                let addr = cpu.pc;
                cpu.pc += 1;
                EffectiveAddress::Memory(addr)
            }
            Mode::ZeroPage => {
                let addr = cpu.fetch_byte(bus) as u16;
                EffectiveAddress::Memory(addr)
            }
            Mode::ZeroPageX => {
                let addr = u8::wrapping_add(cpu.fetch_byte(bus), cpu.x) as u16;
                EffectiveAddress::Memory(addr)
            }
            Mode::ZeroPageY => {
                let addr = u8::wrapping_add(cpu.fetch_byte(bus), cpu.y) as u16;
                EffectiveAddress::Memory(addr)
            }
        }
    }
}