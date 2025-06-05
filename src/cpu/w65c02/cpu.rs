use bitflags::bitflags;

use crate::cpu::w65c02::{inst, Bus};


const VECTOR_RESET: u16 = 0xFFFC;
const VECTOR_IRQ: u16 = 0xFFFE;
const VECTOR_NMI: u16 = 0xFFFA;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Flags: u8 {
        const CARRY     = 0b0000_0001;
        const ZERO      = 0b0000_0010;
        const INTERRUPT = 0b0000_0100;
        const DECIMAL   = 0b0000_1000;
        const BREAK     = 0b0001_0000;
        const UNUSED    = 0b0010_0000;
        const OVERFLOW  = 0b0100_0000;
        const NEGATIVE  = 0b1000_0000;
    }
}

/// Wester Digital W65C02 CPU
pub struct CPU {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub sp: u8,
    pub status: Flags,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: 0,
            status: Flags::empty(),
        }
    }

    pub fn reset(&mut self, bus: &mut impl Bus) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFF;
        self.status = Flags::INTERRUPT;

        self.pc = bus.mem_read_word(VECTOR_RESET);
    }

    pub fn exec<B: Bus>(&mut self, bus: &mut B) -> usize {
        let inst = inst::decode(self, bus);
        (inst.handler)(self, bus, &inst)
    }

    pub fn fetch_byte(&mut self, bus: &mut impl Bus) -> u8 {
        let byte = bus.mem_read(self.pc);
        self.pc += 1;
        byte
    }

    pub fn fetch_word(&mut self, bus: &mut impl Bus) -> u16 {
        let word = bus.mem_read_word(self.pc);
        self.pc += 2;
        word
    }
    
    pub fn push_byte(&mut self, bus: &mut impl Bus, value: u8) {
        bus.mem_write(0x0100 + self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    pub fn pop_byte(&mut self, bus: &mut impl Bus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        bus.mem_read(0x0100 + self.sp as u16)
    }

    pub fn push_word(&mut self, bus: &mut impl Bus, value: u16) {
        self.push_byte(bus, (value >> 8) as u8);
        self.push_byte(bus, value as u8);
    }

    pub fn pop_word(&mut self, bus: &mut impl Bus) -> u16 {
        let low = self.pop_byte(bus);
        let high = self.pop_byte(bus);
        ((high as u16) << 8) | (low as u16)
    }
    
    pub fn zeropage_read_byte(&mut self, bus: &mut impl Bus, addr: u8, offset: u8) -> u8 {
        bus.mem_read(u8::wrapping_add(addr, offset) as u16)
    }

    pub fn zeropage_write_byte(&mut self, bus: &mut impl Bus, addr: u8, offset: u8, val: u8) {
        bus.mem_write(u8::wrapping_add(addr, offset) as u16, val);
    }

    pub fn zeropage_read_word(&mut self, bus: &mut impl Bus, addr: u8, offset: u8) -> u16 {
        bus.mem_read_word_page_wrap(u8::wrapping_add(addr, offset) as u16)
    }

    pub fn zeropage_write_word(&mut self, bus: &mut impl Bus, addr: u8, offset: u8, val: u16) {
        bus.mem_write_word_page_wrap(u8::wrapping_add(addr, offset) as u16, val);
    }

}