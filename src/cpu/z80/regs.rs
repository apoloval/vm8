use bus::Addr16;

pub trait Register<T> {
    fn read(&self, regs: &Registers) -> T;
    fn write(&self, regs: &mut Registers, val: T);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reg8 { A, B, C, D, E }

impl Register<u8> for Reg8 {
    fn read(&self, regs: &Registers) -> u8 {
        match self {
            Reg8::A => (regs.af >> 8) as u8,
            Reg8::B => (regs.bc >> 8) as u8,
            Reg8::C => (regs.bc) as u8,
            Reg8::D => (regs.de >> 8) as u8,
            Reg8::E => (regs.de) as u8,
        }
    }

    fn write(&self, regs: &mut Registers, val: u8) {
        match self {
            Reg8::A => regs.af = (regs.af & 0x00ff) | ((val as u16) << 8),
            Reg8::B => regs.bc = (regs.bc & 0x00ff) | ((val as u16) << 8),
            Reg8::C => regs.bc = (regs.bc & 0xff00) | (val as u16),
            Reg8::D => regs.de = (regs.de & 0x00ff) | ((val as u16) << 8),
            Reg8::E => regs.de = (regs.de & 0xff00) | (val as u16),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reg16 { AF, BC, DE }

impl Register<u16> for Reg16 {
    fn read(&self, regs: &Registers) -> u16 {
        match self {
            Reg16::AF => regs.af,
            Reg16::BC => regs.bc,
            Reg16::DE => regs.de,
        }
    }

    fn write(&self, regs: &mut Registers, val: u16) {
        match self {
            Reg16::AF => regs.af = val,
            Reg16::BC => regs.bc = val,
            Reg16::DE => regs.de = val,
        }
    }
}

pub struct Registers {
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub pc: u16,
}

impl Registers {
    pub fn pc(&self) -> Addr16 {
        return Addr16::from(self.pc)
    }

    pub fn inc_pc(&mut self, val: usize) {
        self.pc += val as u16
    }
}