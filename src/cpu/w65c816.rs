mod bus;
mod ev;
mod int;
mod addr;
mod reg;
mod status;
#[cfg(test)] mod assert;

pub use bus::{Addr, AddrWrap, Bus};
pub use ev::{Event, Reporter, NullReporter};

use crate::utils::bcd;

use self::status::Flag;
#[cfg(test)] use std::str::FromStr;

#[derive(Default)]
pub struct CPU {
    cycles: u64,
    regs: reg::Bank,
}

impl CPU {
    pub fn reset<B: Bus>(&mut self, bus: &mut B) {
        self.regs.reset(bus);
    }

    pub fn step(&mut self, bus: &mut impl Bus, rep: &mut impl Reporter) {
        self.exec_next_instruction(bus, rep);
    }

    fn read_direct_byte(&self, bus: &impl Bus, dir: u8, idx: u16) -> u8 {
        self.read_direct_word(bus, dir, idx) as u8
    }

    fn read_direct_word(&self, bus: &impl Bus, dir: u8, idx: u16) -> u16 {
        let (addr, wrap) = self.direct_addr(dir, idx);
        bus.read_word(addr, wrap)
}

    fn read_direct_ptr(&self, bus: &impl Bus, dir: u8, idx: u16) -> Addr {
        Addr::from(
            self.regs.dbr(), 
            self.read_direct_word(bus, dir, idx)
        )
    }

    fn read_direct_ptr_long(&self, bus: &impl Bus, dir: u8, idx: u16) -> Addr {    
        Addr::from(
            self.read_direct_byte(bus, dir, idx.wrapping_add(2)),
            self.read_direct_word(bus, dir, idx)
        )
    }

    fn read_stack_word(&self, bus: &impl Bus, idx: u16) -> u16 {
        let wrap = if self.regs.mode_is_emulated() { AddrWrap::Byte } 
        else { AddrWrap::Word };

        let addr = Addr::from(0, self.regs.sp())
            .wrapping_add(idx, wrap);
        bus.read_word(addr, wrap)
    }

    fn fetch_pc_byte(&self, bus: &mut impl Bus, offset: u16) -> u8 {
        let addr = Addr::from(self.regs.pbr(), self.regs.pc())
            .wrapping_add(offset, AddrWrap::Word);
        bus.read_byte(addr)
    }

    fn fetch_pc_word(&self, bus: &mut impl Bus, offset: u16) -> u16 {
        let addr = Addr::from(self.regs.pbr(), self.regs.pc())
            .wrapping_add(offset, AddrWrap::Word);
        bus.read_word(addr, AddrWrap::Word)
    }

    fn write_direct_byte(&mut self, bus: &mut impl Bus, dir: u8, idx: u16, value: u8) {
        let (addr, _) = self.direct_addr(dir, idx);
        bus.write_byte(addr, value);
    }

    fn write_direct_word(&mut self, bus: &mut impl Bus, dir: u8, idx: u16, value: u16) {
        let (addr, wrap) = self.direct_addr(dir, idx);
        bus.write_word(addr, wrap, value);
    }

    fn push_byte<B: Bus>(&mut self, bus: &mut B, value: u8) {
        bus.write_byte(Addr::from(0, self.regs.sp()), value);
        self.regs.sp_dec(1);
    }

    fn push_word<B: Bus>(&mut self, bus: &mut B, value: u16) {
        self.push_byte(bus, (value >> 8) as u8);
        self.push_byte(bus, value as u8);
    }

    fn push_pc<B: Bus>(&mut self, bus: &mut B) {
        if self.regs.mode_is_native() {
            self.push_byte(bus, self.regs.pbr());
        }
        self.push_word(bus, self.regs.pc());
    }

    fn direct_addr(&self, dir: u8, idx: u16) -> (Addr, AddrWrap) {
        let wrap = if self.regs.mode_is_emulated() && self.regs.dl() == 0 {
            AddrWrap::Byte
        } else {
            AddrWrap::Word
        };
        (
            Addr::from(0, self.regs.dp())
                .wrapping_add(dir, wrap)
                .wrapping_add(idx as u8, wrap),
            wrap,
        )
    }

    fn update_status_zero(&mut self, result: u16, flag_8bit: Flag) {                
        if self.regs.status_flag_is_set(flag_8bit) {
            self.regs.set_status_flag(Flag::Z, result & 0x00FF == 0);
        } else {
            self.regs.set_status_flag(Flag::Z, result == 0);
        }
    }

    fn update_status_negative(&mut self, result: u16, flag_8bit: Flag) {                
        if self.regs.status_flag_is_set(flag_8bit) {
            self.regs.set_status_flag(Flag::N, result & 0x0080 != 0);
        } else {
            self.regs.set_status_flag(Flag::N, result & 0x8000 != 0);
        }
    }

    fn update_status_negative_second(&mut self, result: u16, flag_8bit: Flag) {                
        if self.regs.status_flag_is_set(flag_8bit) {
            self.regs.set_status_flag(Flag::V, result & 0x0040 != 0);
        } else {
            self.regs.set_status_flag(Flag::V, result & 0x4000 != 0);
        }
    }

    fn update_status_carry_arithmetic(&mut self, prev: u16, result: u16, flag_8bit: Flag) {                
        if self.regs.status_flag_is_set(flag_8bit) {
            self.regs.set_status_flag(Flag::C, (result as u8) < (prev as u8));
        } else {
            self.regs.set_status_flag(Flag::C, (result as u16) < (prev as u16));
        }  
    }

    fn update_status_carry_shift_left(&mut self, prev: u16, flag_8bit: Flag) {                
        if self.regs.status_flag_is_set(flag_8bit) {
            self.regs.set_status_flag(Flag::C, prev & 0x0080 != 0);
        } else {
            self.regs.set_status_flag(Flag::C, prev & 0x8000 != 0);
        }  
    }

    fn update_status_overflow(&mut self, prev: u16, result: u16, flag_8bit: Flag) {                
        if self.regs.status_flag_is_set(flag_8bit) {
            self.regs.set_status_flag(Flag::V, (result as i8) < (prev as i8));
        } else {
            self.regs.set_status_flag(Flag::V, (result as i16) < (prev as i16));
        }    
    }

    fn update_status_underflow(&mut self, prev: u16, result: u16, flag_8bit: Flag) {                
        if self.regs.status_flag_is_set(flag_8bit) {
            self.regs.set_status_flag(Flag::V, (result as i8) > (prev as i8));
        } else {
            self.regs.set_status_flag(Flag::V, (result as i16) > (prev as i16));
        }    
    }
}

/**********************************/
/* Instruction set implementation */
/**********************************/
impl CPU {
    pub fn exec_next_instruction(&mut self, bus: &mut impl Bus, rep: &mut impl Reporter) {
        match self.fetch_pc_byte(bus, 0) {
            0x00 => 
                // BRK
                self.brk(bus, rep),
            0x01 => {
                // ORA (d,X)
                let dir = self.fetch_pc_byte(bus, 1);
                self.ora(bus, addr::Mode::DirectIndexedIndirect(dir), rep)
            },
            0x03 => {
                // ORA d,S
                let rel = self.fetch_pc_byte(bus, 1);
                self.ora(bus, addr::Mode::StackRelative(rel), rep)
            },
            0x04 => {
                // TSB d
                let dir = self.fetch_pc_byte(bus, 1);
                self.tsb(bus, addr::Mode::Direct(dir), rep)
            },
            0x05 => {
                // ORA d
                let dir = self.fetch_pc_byte(bus, 1);
                self.ora(bus, addr::Mode::Direct(dir), rep)
            },
            0x06 => {
                // ASL d
                let dir = self.fetch_pc_byte(bus, 1);
                self.asl(bus, addr::Mode::Direct(dir), rep)
            },
            0x07 => {
                // ORA [d]
                let dir = self.fetch_pc_byte(bus, 1);
                self.ora(bus, addr::Mode::DirectIndirectLong(dir), rep)
            },
            0x09 => {
                // ORA #i
                let imm = self.fetch_pc_word(bus, 1);
                self.ora(bus, addr::Mode::Immediate(imm), rep)
            },
            0x0A => {
                // ASL
                self.asl(bus, addr::Mode::Accumulator, rep)
            },
            0x0C => {
                // TSB a
                let abs = self.fetch_pc_word(bus, 1);
                self.tsb(bus, addr::Mode::Absolute(abs), rep)
            },
            0x0D => {
                // ORA a
                let abs = self.fetch_pc_word(bus, 1);
                self.ora(bus, addr::Mode::Absolute(abs), rep)
            },
            0x0E => {
                // ASL a
                let abs = self.fetch_pc_word(bus, 1);
                self.asl(bus, addr::Mode::Absolute(abs), rep)
            },
            0x0F => {
                // ORA al
                let abs = self.fetch_pc_word(bus, 1);
                let bank = self.fetch_pc_byte(bus, 3);
                self.ora(bus, addr::Mode::AbsoluteLong(bank, abs), rep)
            },
            0x11 => {
                // ORA (d),Y
                let dir = self.fetch_pc_byte(bus, 1);
                self.ora(bus, addr::Mode::DirectIndirectIndexed(dir), rep)
            },
            0x12 => {
                // ORA (d)
                let dir = self.fetch_pc_byte(bus, 1);
                self.ora(bus, addr::Mode::DirectIndirect(dir), rep)
            },
            0x13 => {
                // ORA (d,S),Y
                let rel = self.fetch_pc_byte(bus, 1);
                self.ora(bus, addr::Mode::StackRelativeIndirectIndexed(rel), rep)
            },
            0x14 => {
                // TRB d
                let dir = self.fetch_pc_byte(bus, 1);
                self.trb(bus, addr::Mode::Direct(dir), rep)
            },
            0x15 => {
                // ORA d,X
                let dir = self.fetch_pc_byte(bus, 1);
                self.ora(bus, addr::Mode::DirectIndexedX(dir), rep)
            },
            0x16 => {
                // ASL d,X
                let dir = self.fetch_pc_byte(bus, 1);
                self.asl(bus, addr::Mode::DirectIndexedX(dir), rep)
            },
            0x17 => {
                // ORA [d],Y
                let dir = self.fetch_pc_byte(bus, 1);
                self.ora(bus, addr::Mode::DirectIndirectLongIndexed(dir), rep)
            },
            0x19 => {
                // ORA a,Y
                let abs = self.fetch_pc_word(bus, 1);
                self.ora(bus, addr::Mode::AbsoluteIndexedY(abs), rep)
            },
            0x1A => {
                // INC
                self.inc(bus, addr::Mode::Accumulator, rep)
            },
            0x1C => {
                // TRB a
                let abs = self.fetch_pc_word(bus, 1);
                self.trb(bus, addr::Mode::Absolute(abs), rep)
            },
            0x1D => {
                // ORA a,X
                let abs = self.fetch_pc_word(bus, 1);
                self.ora(bus, addr::Mode::AbsoluteIndexedX(abs), rep)
            },
            0x1E => {
                // ASL a,X
                let abs = self.fetch_pc_word(bus, 1);
                self.asl(bus, addr::Mode::AbsoluteIndexedX(abs), rep)
            },
            0x1F => {
                // ORA al,X
                let abs = self.fetch_pc_word(bus, 1);
                let bank = self.fetch_pc_byte(bus, 3);
                self.ora(bus, addr::Mode::AbsoluteLongIndexed(bank, abs), rep)
            },
            0x21 => {
                // AND (d,X)
                let dir = self.fetch_pc_byte(bus, 1);
                self.and(bus, addr::Mode::DirectIndexedIndirect(dir), rep)
            },
            0x23 => {
                // AND d,S
                let rel = self.fetch_pc_byte(bus, 1);
                self.and(bus, addr::Mode::StackRelative(rel), rep)
            },
            0x24 => {
                // BIT d
                let dir = self.fetch_pc_byte(bus, 1);
                self.bit(bus, addr::Mode::Direct(dir), rep)
            },
            0x25 => {
                // AND d
                let dir = self.fetch_pc_byte(bus, 1);
                self.and(bus, addr::Mode::Direct(dir), rep)
            },
            0x27 => {
                // AND [d]
                let dir = self.fetch_pc_byte(bus, 1);
                self.and(bus, addr::Mode::DirectIndirectLong(dir), rep)
            },
            0x29 => {
                // AND #i
                let imm = self.fetch_pc_word(bus, 1);
                self.and(bus, addr::Mode::Immediate(imm), rep)
            },
            0x2C => {
                // BIT a
                let abs = self.fetch_pc_word(bus, 1);
                self.bit(bus, addr::Mode::Absolute(abs), rep)
            },
            0x2D => {
                // AND a
                let abs = self.fetch_pc_word(bus, 1);
                self.and(bus, addr::Mode::Absolute(abs), rep)
            },
            0x2F => {
                // AND al
                let abs = self.fetch_pc_word(bus, 1);
                let bank = self.fetch_pc_byte(bus, 3);
                self.and(bus, addr::Mode::AbsoluteLong(bank, abs), rep)
            },
            0x31 => {
                // AND (d),Y
                let dir = self.fetch_pc_byte(bus, 1);
                self.and(bus, addr::Mode::DirectIndirectIndexed(dir), rep)
            },
            0x32 => {
                // AND (d)
                let dir = self.fetch_pc_byte(bus, 1);
                self.and(bus, addr::Mode::DirectIndirect(dir), rep)
            },
            0x33 => {
                // AND (d,S),Y
                let rel = self.fetch_pc_byte(bus, 1);
                self.and(bus, addr::Mode::StackRelativeIndirectIndexed(rel), rep)
            },
            0x34 => {
                // BIT d,X
                let dir = self.fetch_pc_byte(bus, 1);
                self.bit(bus, addr::Mode::DirectIndexedX(dir), rep)
            },
            0x35 => {
                // AND d,X
                let dir = self.fetch_pc_byte(bus, 1);
                self.and(bus, addr::Mode::DirectIndexedX(dir), rep)
            },
            0x37 => {
                // AND [d],Y
                let dir = self.fetch_pc_byte(bus, 1);
                self.and(bus, addr::Mode::DirectIndirectLongIndexed(dir), rep)
            },
            0x39 => {
                // AND a,Y
                let abs = self.fetch_pc_word(bus, 1);
                self.and(bus, addr::Mode::AbsoluteIndexedY(abs), rep)
            },
            0x3A => {
                // DEC
                self.dec(bus, addr::Mode::Accumulator, rep)
            },
            0x3C => {
                // BIT a,X
                let abs = self.fetch_pc_word(bus, 1);
                self.bit(bus, addr::Mode::AbsoluteIndexedX(abs), rep)
            },
            0x3D => {
                // AND a,X
                let abs = self.fetch_pc_word(bus, 1);
                self.and(bus, addr::Mode::AbsoluteIndexedX(abs), rep)
            },
            0x3F => {
                // AND al,X
                let abs = self.fetch_pc_word(bus, 1);
                let bank = self.fetch_pc_byte(bus, 3);
                self.and(bus, addr::Mode::AbsoluteLongIndexed(bank, abs), rep)
            },
            0x41 => {
                // EOR (d,X)
                let dir = self.fetch_pc_byte(bus, 1);
                self.eor(bus, addr::Mode::DirectIndexedIndirect(dir), rep)
            },
            0x43 => {
                // EOR d,S
                let rel = self.fetch_pc_byte(bus, 1);
                self.eor(bus, addr::Mode::StackRelative(rel), rep)
            },
            0x45 => {
                // EOR d
                let dir: u8 = self.fetch_pc_byte(bus, 1);
                self.eor(bus, addr::Mode::Direct(dir), rep)
            },
            0x47 => {
                // EOR [d]
                let dir = self.fetch_pc_byte(bus, 1);
                self.eor(bus, addr::Mode::DirectIndirectLong(dir), rep)
            },
            0x49 => {
                // EOR #i
                let imm = self.fetch_pc_word(bus, 1);
                self.eor(bus, addr::Mode::Immediate(imm), rep)
            },
            0x4D => {
                // EOR a
                let abs = self.fetch_pc_word(bus, 1);
                self.eor(bus, addr::Mode::Absolute(abs), rep)
            },
            0x4F => {
                // EOR al
                let abs = self.fetch_pc_word(bus, 1);
                let bank = self.fetch_pc_byte(bus, 3);
                self.eor(bus, addr::Mode::AbsoluteLong(bank, abs), rep)
            },
            0x51 => {
                // EOR (d),Y
                let dir = self.fetch_pc_byte(bus, 1);
                self.eor(bus, addr::Mode::DirectIndirectIndexed(dir), rep)
            },
            0x52 => {
                // EOR (d)
                let dir = self.fetch_pc_byte(bus, 1);
                self.eor(bus, addr::Mode::DirectIndirect(dir), rep)
            },
            0x53 => {
                // EOR (d,S),Y
                let rel = self.fetch_pc_byte(bus, 1);
                self.eor(bus, addr::Mode::StackRelativeIndirectIndexed(rel), rep)
            },
            0x55 => {
                // EOR d,X
                let dir = self.fetch_pc_byte(bus, 1);
                self.eor(bus, addr::Mode::DirectIndexedX(dir), rep)
            },
            0x57 => {
                // EOR [d],Y
                let dir = self.fetch_pc_byte(bus, 1);
                self.eor(bus, addr::Mode::DirectIndirectLongIndexed(dir), rep)
            },
            0x59 => {
                // EOR a,Y
                let abs = self.fetch_pc_word(bus, 1);
                self.eor(bus, addr::Mode::AbsoluteIndexedY(abs), rep)
            },
            0x5D => {
                // EOR a,X
                let abs = self.fetch_pc_word(bus, 1);
                self.eor(bus, addr::Mode::AbsoluteIndexedX(abs), rep)
            },
            0x5F => {
                // EOR al,X
                let abs = self.fetch_pc_word(bus, 1);
                let bank = self.fetch_pc_byte(bus, 3);
                self.eor(bus, addr::Mode::AbsoluteLongIndexed(bank, abs), rep)
            },            
            0x61 => {
                // ADC (d,X)
                let dir = self.fetch_pc_byte(bus, 1);
                self.adc(bus, addr::Mode::DirectIndexedIndirect(dir), rep)
            },
            0x63 => {
                // ADC d,S
                let rel = self.fetch_pc_byte(bus, 1);
                self.adc(bus, addr::Mode::StackRelative(rel), rep)
            },
            0x65 => {
                // ADC d
                let dir: u8 = self.fetch_pc_byte(bus, 1);
                self.adc(bus, addr::Mode::Direct(dir), rep)
            },
            0x67 => {
                // ADC [d]
                let dir = self.fetch_pc_byte(bus, 1);
                self.adc(bus, addr::Mode::DirectIndirectLong(dir), rep)
            },
            0x69 => {
                // ADC #i
                let imm = self.fetch_pc_word(bus, 1);
                self.adc(bus, addr::Mode::Immediate(imm), rep)
            },
            0x6D => {
                // ADC a
                let abs = self.fetch_pc_word(bus, 1);
                self.adc(bus, addr::Mode::Absolute(abs), rep)
            },
            0x6F => {
                // ADC al
                let abs = self.fetch_pc_word(bus, 1);
                let bank = self.fetch_pc_byte(bus, 3);
                self.adc(bus, addr::Mode::AbsoluteLong(bank, abs), rep)
            },
            0x71 => {
                // ADC (d),Y
                let dir = self.fetch_pc_byte(bus, 1);
                self.adc(bus, addr::Mode::DirectIndirectIndexed(dir), rep)
            },
            0x72 => {
                // ADC (d)
                let dir = self.fetch_pc_byte(bus, 1);
                self.adc(bus, addr::Mode::DirectIndirect(dir), rep)
            },
            0x73 => {
                // ADC (d,S),Y
                let rel = self.fetch_pc_byte(bus, 1);
                self.adc(bus, addr::Mode::StackRelativeIndirectIndexed(rel), rep)
            },
            0x75 => {
                // ADC d,X
                let dir = self.fetch_pc_byte(bus, 1);
                self.adc(bus, addr::Mode::DirectIndexedX(dir), rep)
            },
            0x77 => {
                // ADC [d],Y
                let dir = self.fetch_pc_byte(bus, 1);
                self.adc(bus, addr::Mode::DirectIndirectLongIndexed(dir), rep)
            },
            0x79 => {
                // ADC a,Y
                let abs = self.fetch_pc_word(bus, 1);
                self.adc(bus, addr::Mode::AbsoluteIndexedY(abs), rep)
            },
            0x7D => {
                // ADC a,X
                let abs = self.fetch_pc_word(bus, 1);
                self.adc(bus, addr::Mode::AbsoluteIndexedX(abs), rep)
            },
            0x7F => {
                // ADC al,X
                let abs = self.fetch_pc_word(bus, 1);
                let bank = self.fetch_pc_byte(bus, 3);
                self.adc(bus, addr::Mode::AbsoluteLongIndexed(bank, abs), rep)
            },
            0x88 => {
                // DEY
                self.dey(rep)
            },
            0x89 => {
                // BIT #i
                let imm = self.fetch_pc_word(bus, 1);
                self.bit(bus, addr::Mode::Immediate(imm), rep)
            },
            0xC0 => {
                // CPY #i
                let imm = self.fetch_pc_word(bus, 1);
                self.cpy(bus, addr::Mode::Immediate(imm), rep)
            },
            0xC1 => {
                // CMP (d,X)
                let dir = self.fetch_pc_byte(bus, 1);
                self.cmp(bus, addr::Mode::DirectIndexedIndirect(dir), rep)
            },
            0xC3 => {
                // CMP d,S
                let rel = self.fetch_pc_byte(bus, 1);
                self.cmp(bus, addr::Mode::StackRelative(rel), rep)
            },
            0xC4 => {
                // CPY d
                let dir: u8 = self.fetch_pc_byte(bus, 1);
                self.cpy(bus, addr::Mode::Direct(dir), rep)
            },
            0xC5 => {
                // CMP d
                let dir: u8 = self.fetch_pc_byte(bus, 1);
                self.cmp(bus, addr::Mode::Direct(dir), rep)
            },
            0xC6 => {
                // DEC d
                let dir = self.fetch_pc_byte(bus, 1);
                self.dec(bus, addr::Mode::Direct(dir), rep)
            },
            0xC7 => {
                // CMP [d]
                let dir = self.fetch_pc_byte(bus, 1);
                self.cmp(bus, addr::Mode::DirectIndirectLong(dir), rep)
            },
            0xC8 => {
                // INY
                self.iny(rep)
            },
            0xC9 => {
                // CMP #i
                let imm = self.fetch_pc_word(bus, 1);
                self.cmp(bus, addr::Mode::Immediate(imm), rep)
            },
            0xCA => {
                // DEX
                self.dex(rep)
            },
            0xCC => {
                // CPY a
                let abs = self.fetch_pc_word(bus, 1);
                self.cpy(bus, addr::Mode::Absolute(abs), rep)
            },
            0xCD => {
                // CMP a
                let abs = self.fetch_pc_word(bus, 1);
                self.cmp(bus, addr::Mode::Absolute(abs), rep)
            },
            0xCE => {
                // DEC a
                let abs = self.fetch_pc_word(bus, 1);
                self.dec(bus, addr::Mode::Absolute(abs), rep)
            },
            0xCF => {
                // CMP al
                let abs = self.fetch_pc_word(bus, 1);
                let bank = self.fetch_pc_byte(bus, 3);
                self.cmp(bus, addr::Mode::AbsoluteLong(bank, abs), rep)
            },
            0xD1 => {
                // CMP (d),Y
                let dir = self.fetch_pc_byte(bus, 1);
                self.cmp(bus, addr::Mode::DirectIndirectIndexed(dir), rep)
            },
            0xD2 => {
                // CMP (d)
                let dir = self.fetch_pc_byte(bus, 1);
                self.cmp(bus, addr::Mode::DirectIndirect(dir), rep)
            },
            0xD3 => {
                // CMP (d,S),Y
                let rel = self.fetch_pc_byte(bus, 1);
                self.cmp(bus, addr::Mode::StackRelativeIndirectIndexed(rel), rep)
            },
            0xD5 => {
                // CMP d,X
                let dir = self.fetch_pc_byte(bus, 1);
                self.cmp(bus, addr::Mode::DirectIndexedX(dir), rep)
            },
            0xD6 => {
                // DEC d,X
                let dir = self.fetch_pc_byte(bus, 1);
                self.dec(bus, addr::Mode::DirectIndexedX(dir), rep)
            },
            0xD7 => {
                // CMP [d],Y
                let dir = self.fetch_pc_byte(bus, 1);
                self.cmp(bus, addr::Mode::DirectIndirectLongIndexed(dir), rep)
            },
            0xD9 => {
                // CMP a,Y
                let abs = self.fetch_pc_word(bus, 1);
                self.cmp(bus, addr::Mode::AbsoluteIndexedY(abs), rep)
            },
            0xDD => {
                // CMP a,X
                let abs = self.fetch_pc_word(bus, 1);
                self.cmp(bus, addr::Mode::AbsoluteIndexedX(abs), rep)
            },
            0xDE => {
                // DEC a,X
                let abs = self.fetch_pc_word(bus, 1);
                self.dec(bus, addr::Mode::AbsoluteIndexedX(abs), rep)
            },
            0xDF => {
                // CMP al,X
                let abs = self.fetch_pc_word(bus, 1);
                let bank = self.fetch_pc_byte(bus, 3);
                self.cmp(bus, addr::Mode::AbsoluteLongIndexed(bank, abs), rep)
            },
            0xE0 => {
                // CPX #i
                let imm = self.fetch_pc_word(bus, 1);
                self.cpx(bus, addr::Mode::Immediate(imm), rep)
            },
            0xE1 => {
                // SBC (d,X)
                let dir = self.fetch_pc_byte(bus, 1);
                self.sbc(bus, addr::Mode::DirectIndexedIndirect(dir), rep)
            },
            0xE3 => {
                // SBC d,S
                let rel = self.fetch_pc_byte(bus, 1);
                self.sbc(bus, addr::Mode::StackRelative(rel), rep)
            },
            0xE4 => {
                // CPX d
                let dir: u8 = self.fetch_pc_byte(bus, 1);
                self.cpx(bus, addr::Mode::Direct(dir), rep)
            },
            0xE5 => {
                // SBC d
                let dir: u8 = self.fetch_pc_byte(bus, 1);
                self.sbc(bus, addr::Mode::Direct(dir), rep)
            },
            0xE6 => {
                // INC d
                let dir = self.fetch_pc_byte(bus, 1);
                self.inc(bus, addr::Mode::Direct(dir), rep)
            },
            0xE7 => {
                // SBC [d]
                let dir = self.fetch_pc_byte(bus, 1);
                self.sbc(bus, addr::Mode::DirectIndirectLong(dir), rep)
            },
            0xE8 => {
                // INX
                self.inx(rep)
            },
            0xE9 => {
                // SBC #i
                let imm = self.fetch_pc_word(bus, 1);
                self.sbc(bus, addr::Mode::Immediate(imm), rep)
            },
            0xEC => {
                // CPX a
                let abs = self.fetch_pc_word(bus, 1);
                self.cpx(bus, addr::Mode::Absolute(abs), rep)
            },
            0xED => {
                // SBC a
                let abs = self.fetch_pc_word(bus, 1);
                self.sbc(bus, addr::Mode::Absolute(abs), rep)
            },
            0xEE => {
                // INC a
                let abs = self.fetch_pc_word(bus, 1);
                self.inc(bus, addr::Mode::Absolute(abs), rep)
            },
            0xEF => {
                // SBC al
                let abs = self.fetch_pc_word(bus, 1);
                let bank = self.fetch_pc_byte(bus, 3);
                self.sbc(bus, addr::Mode::AbsoluteLong(bank, abs), rep)
            },
            0xF1 => {
                // SBC (d),Y
                let dir = self.fetch_pc_byte(bus, 1);
                self.sbc(bus, addr::Mode::DirectIndirectIndexed(dir), rep)
            },
            0xF2 => {
                // SBC (d)
                let dir = self.fetch_pc_byte(bus, 1);
                self.sbc(bus, addr::Mode::DirectIndirect(dir), rep)
            },
            0xF3 => {
                // SBC (d,S),Y
                let rel = self.fetch_pc_byte(bus, 1);
                self.sbc(bus, addr::Mode::StackRelativeIndirectIndexed(rel), rep)
            },
            0xF5 => {
                // SBC d,X
                let dir = self.fetch_pc_byte(bus, 1);
                self.sbc(bus, addr::Mode::DirectIndexedX(dir), rep)
            },
            0xF6 => {
                // INC d,X
                let dir = self.fetch_pc_byte(bus, 1);
                self.inc(bus, addr::Mode::DirectIndexedX(dir), rep)
            },
            0xF7 => {
                // SBC [d],Y
                let dir = self.fetch_pc_byte(bus, 1);
                self.sbc(bus, addr::Mode::DirectIndirectLongIndexed(dir), rep)
            },
            0xF9 => {
                // SBC a,Y
                let abs = self.fetch_pc_word(bus, 1);
                self.sbc(bus, addr::Mode::AbsoluteIndexedY(abs), rep)
            },
            0xFD => {
                // SBC a,X
                let abs = self.fetch_pc_word(bus, 1);
                self.sbc(bus, addr::Mode::AbsoluteIndexedX(abs), rep)
            },
            0xFE => {
                // INC a,X
                let abs = self.fetch_pc_word(bus, 1);
                self.inc(bus, addr::Mode::AbsoluteIndexedX(abs), rep)
            },
            0xFF => {
                // SBC al,X
                let abs = self.fetch_pc_word(bus, 1);
                let bank = self.fetch_pc_byte(bus, 3);
                self.sbc(bus, addr::Mode::AbsoluteLongIndexed(bank, abs), rep)
            },
            _ => unimplemented!()
        }
    }

    fn adc(&mut self, bus: &mut impl Bus, mode: addr::Mode, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("ADC"),
            operands: format!("{}", mode),
        });

        let read = mode.read(self, bus);
        let prev = self.regs.a();

        let carry = if self.regs.status_flag_is_set(Flag::C) { 1 } else { 0 };
        let result = if self.regs.status_flag_is_set(Flag::D) {
            bcd::add_word(
                bcd::add_word(prev, read.val),
                carry,
            )
        } else {
            prev.wrapping_add(read.val).wrapping_add(carry)
        };

        addr::Mode::Accumulator.write(self, bus, result);
        self.update_status_zero(result, Flag::M);
        self.update_status_negative(result, Flag::M);
        self.update_status_carry_arithmetic(prev, result, Flag::M);
        self.update_status_overflow(prev, result, Flag::M);
        self.regs.pc_inc(read.prog_bytes);
        self.cycles += read.cycles;
    }

    fn and(&mut self, bus: &mut impl Bus, mode: addr::Mode, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("AND"), 
            operands: format!("{}", mode),
        });

        let read = mode.read(self, bus);
        let result = self.regs.a() & read.val;
        
        addr::Mode::Accumulator.write(self, bus, result);
        self.update_status_zero(result, Flag::M);
        self.update_status_negative(result, Flag::M);
        self.regs.pc_inc(read.prog_bytes);
        self.cycles += read.cycles;
    }

    fn asl(&mut self, bus: &mut impl Bus, mode: addr::Mode, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("ASL"), 
            operands: format!("{}", mode),
        });

        let read = mode.read(self, bus);
        let result = read.val << 1;
        
        mode.write(self, bus, result);
        self.update_status_zero(result, Flag::M);
        self.update_status_negative(result, Flag::M);
        self.update_status_carry_shift_left(read.val, Flag::M);
        self.regs.pc_inc(read.prog_bytes);
        self.cycles += read.cycles;
    }

    fn bit(&mut self, bus: &mut impl Bus, mode: addr::Mode, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("BIT"),
            operands: format!("{}", mode),
        });

        let read = mode.read(self, bus);
        let result = self.regs.a() & read.val;
                        
        if !matches!(mode, addr::Mode::Immediate(_)){
            self.update_status_negative(read.val, Flag::M);
            self.update_status_negative_second(read.val, Flag::M);
        }
        self.update_status_zero(result, Flag::M);
        self.regs.pc_inc(read.prog_bytes);
        self.cycles += read.cycles;
    }

    fn brk(&mut self, bus: &mut impl Bus, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("BRK"), 
            operands: String::from(""),
        });

        if self.regs.mode_is_emulated() {
            self.regs.set_status_flag(status::Flag::B, true);
        }
        self.regs.pc_inc(2);
        self.push_pc(bus);
        self.push_byte(bus, self.regs.p());

        let vector = 
            if self.regs.mode_is_emulated() { int::VECTOR_EMULATION_IRQBRK }
            else { int::VECTOR_NATIVE_BRK };

        self.regs.pc_jump(bus.read_word(Addr::from(0, vector), AddrWrap::Long));
        self.regs.pbr_set(0);

        self.cycles += 7;
    }

    fn cmp(&mut self, bus: &mut impl Bus, mode: addr::Mode, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("CMP"),
            operands: format!("{}", mode),
        });

        let read = mode.read(self, bus);
        let prev = self.regs.a();
        let result = prev.wrapping_sub(read.val);

        self.update_status_zero(result, Flag::M);
        self.update_status_negative(result, Flag::M);
        self.update_status_carry_arithmetic(prev, result, Flag::M);
        self.regs.pc_inc(read.prog_bytes);
        self.cycles += read.cycles;
    }

    fn cpx(&mut self, bus: &mut impl Bus, mode: addr::Mode, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("CPX"),
            operands: format!("{}", mode),
        });

        let read = mode.read(self, bus);
        let prev = self.regs.x();
        let result = prev.wrapping_sub(read.val);

        self.update_status_zero(result, Flag::M);
        self.update_status_negative(result, Flag::M);
        self.update_status_carry_arithmetic(prev, result, Flag::M);
        self.regs.pc_inc(read.prog_bytes);
        self.cycles += read.cycles;
    }

    fn cpy(&mut self, bus: &mut impl Bus, mode: addr::Mode, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("CPY"),
            operands: format!("{}", mode),
        });

        let read = mode.read(self, bus);
        let prev = self.regs.y();
        let result = prev.wrapping_sub(read.val);

        self.update_status_zero(result, Flag::M);
        self.update_status_negative(result, Flag::M);
        self.update_status_carry_arithmetic(prev, result, Flag::M);
        self.regs.pc_inc(read.prog_bytes);
        self.cycles += read.cycles;
    }

    fn dec(&mut self, bus: &mut impl Bus, mode: addr::Mode, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("DEC"),
            operands: format!("{}", mode),
        });

        let read = mode.read(self, bus);
        let result = read.val.wrapping_sub(1);
        mode.write(self, bus, result);
        self.update_status_negative(result, Flag::M);
        self.update_status_zero(result, Flag::M);
        self.regs.pc_inc(read.prog_bytes);
        self.cycles += read.cycles;
    }

    fn dex(&mut self, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("DEX"),
            operands: String::from(""),
        });

        let read = self.regs.x();
        let result = read.wrapping_sub(1);
        self.regs.x_set(result);
        self.update_status_negative(result, Flag::X);
        self.update_status_zero(result, Flag::X);
        self.regs.pc_inc(1);
        self.cycles += 2
    }

    fn dey(&mut self, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("DEY"),
            operands: String::from(""),
        });

        let read = self.regs.y();
        let result = read.wrapping_sub(1);
        self.regs.y_set(result);
        self.update_status_negative(result, Flag::X);
        self.update_status_zero(result, Flag::X);
        self.regs.pc_inc(1);
        self.cycles += 2
    }

    fn eor(&mut self, bus: &mut impl Bus, mode: addr::Mode, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("EOR"), 
            operands: format!("{}", mode),
        });

        let read = mode.read(self, bus);
        let result = self.regs.a() ^ read.val;
        
        addr::Mode::Accumulator.write(self, bus, result);
        self.update_status_negative(result, Flag::M);
        self.update_status_zero(result, Flag::M);
        self.regs.pc_inc(read.prog_bytes);
        self.cycles += read.cycles;

    }

    fn inc(&mut self, bus: &mut impl Bus, mode: addr::Mode, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("INC"),
            operands: format!("{}", mode),
        });

        let read = mode.read(self, bus);
        let result = read.val.wrapping_add(1);
        mode.write(self, bus, result);
        self.update_status_negative(result, Flag::M);
        self.update_status_zero(result, Flag::M);
        self.regs.pc_inc(read.prog_bytes);
        self.cycles += read.cycles;
    }

    fn inx(&mut self, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("INX"),
            operands: String::from(""),
        });

        let read = self.regs.x();
        let result = read.wrapping_add(1);
        self.regs.x_set(result);
        self.update_status_negative(result, Flag::X);
        self.update_status_zero(result, Flag::X);
        self.regs.pc_inc(1);
        self.cycles += 2
    }

    fn iny(&mut self, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("INY"),
            operands: String::from(""),
        });

        let read = self.regs.y();
        let result = read.wrapping_add(1);
        self.regs.y_set(result);
        self.update_status_negative(result, Flag::X);
        self.update_status_zero(result, Flag::X);
        self.regs.pc_inc(1);
        self.cycles += 2
    }

    fn ora(&mut self, bus: &mut impl Bus, mode: addr::Mode, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("ORA"), 
            operands: format!("{}", mode),
        });

        let read = mode.read(self, bus);
        let result = self.regs.a() | read.val;
        
        addr::Mode::Accumulator.write(self, bus, result);
        self.update_status_negative(result, Flag::M);
        self.update_status_zero(result, Flag::M);
        self.regs.pc_inc(read.prog_bytes);
        self.cycles += read.cycles;

    }

    fn sbc(&mut self, bus: &mut impl Bus, mode: addr::Mode, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("SBC"),
            operands: format!("{}", mode),
        });

        let read = mode.read(self, bus);
        let prev = self.regs.a();

        // Remind that borrow is the negation of the carry flag in W65C816.
        let borrow = if self.regs.status_flag_is_set(Flag::C) { 0 } else { 1 };
        let result = if self.regs.status_flag_is_set(Flag::D) {
            bcd::sub_word(
                bcd::sub_word(prev, read.val),
                borrow,
            )
        } else {
            prev.wrapping_sub(read.val).wrapping_sub(borrow)
        };

        addr::Mode::Accumulator.write(self, bus, result);
        self.update_status_zero(result, Flag::M);
        self.update_status_negative(result, Flag::M);
        self.update_status_carry_arithmetic(prev, result, Flag::M);
        self.update_status_underflow(prev, result, Flag::M);
        self.regs.pc_inc(read.prog_bytes);
        self.cycles += read.cycles;
    }

    fn trb(&mut self, bus: &mut impl Bus, mode: addr::Mode, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("TRB"),
            operands: format!("{}", mode),
        });

        let read = mode.read(self, bus);
        let mask = self.regs.a();
        let result = read.val & !mask;       

        mode.write(self, bus, result);
        self.update_status_zero(read.val & mask, Flag::M);
        self.regs.pc_inc(read.prog_bytes);
        self.cycles += read.cycles;
    }

    fn tsb(&mut self, bus: &mut impl Bus, mode: addr::Mode, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("TSB"),
            operands: format!("{}", mode),
        });

        let read = mode.read(self, bus);
        let mask = self.regs.a();
        let result = read.val | mask;       

        mode.write(self, bus, result);
        self.update_status_zero(read.val & mask, Flag::M);
        self.regs.pc_inc(read.prog_bytes);
        self.cycles += read.cycles;
    }
}

#[cfg(test)]
impl FromStr for CPU {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cpu = CPU::default();
        if s == "" {
            return Ok(cpu);
        }

        for prop in s.split(',') {
            let mut parts = prop.split(':');
            match (parts.next(), parts.next()) {
                (Some("P.C"), Some(val)) => 
                    cpu.regs.set_status_flag(Flag::C, val == "1"),
                (Some("P.Z"), Some(val)) => 
                    cpu.regs.set_status_flag(Flag::Z, val == "1"),
                (Some("P.I"), Some(val)) => 
                    cpu.regs.set_status_flag(Flag::I, val == "1"),
                (Some("P.D"), Some(val)) => 
                    cpu.regs.set_status_flag(Flag::D, val == "1"),
                (Some("P.X"), Some(val)) => 
                    cpu.regs.set_status_flag(Flag::X, val == "1"),
                (Some("P.M"), Some(val)) => 
                    cpu.regs.set_status_flag(Flag::M, val == "1"),
                (Some("P.O"), Some(val)) => 
                    cpu.regs.set_status_flag(Flag::V, val == "1"),
                (Some("P.N"), Some(val)) => 
                    cpu.regs.set_status_flag(Flag::N, val == "1"),
                (Some("P.B"), Some(val)) => 
                    cpu.regs.set_status_flag(Flag::B, val == "1"),
                (Some("P.E"), Some(val)) => 
                    if val == "1" { cpu.regs.set_mode_emulated() } 
                    else { cpu.regs.set_mode_native() }
                (Some("PBR"), Some(value)) => 
                    cpu.regs.pbr_set(u8::from_str_radix(value, 16).unwrap()),
                (Some("DBR"), Some(value)) => 
                    cpu.regs.dbr_set(u8::from_str_radix(value, 16).unwrap()),
                (Some("PC"), Some(value)) => 
                    cpu.regs.pc_jump(u16::from_str_radix(value, 16).unwrap()),
                (Some("SP"), Some(value)) => 
                    cpu.regs.sp_set(u16::from_str_radix(value, 16).unwrap()),
                (Some("DP"), Some(value)) => 
                    cpu.regs.dp_set(u16::from_str_radix(value, 16).unwrap()),
                (Some("P"), Some(value)) => 
                    cpu.regs.p_set(u8::from_str_radix(value, 16).unwrap()),
                (Some("A"), Some(value)) => 
                    cpu.regs.a_set(u16::from_str_radix(value, 16).unwrap()),
                (Some("X"), Some(value)) => 
                    cpu.regs.x_set(u16::from_str_radix(value, 16).unwrap()),
                (Some("Y"), Some(value)) => 
                    cpu.regs.y_set(u16::from_str_radix(value, 16).unwrap()),
                _ => return Err(format!("Invalid property: {}", prop))
            }
        }

        Ok(cpu)
    }

}

#[cfg(test)] mod tests_adc;
#[cfg(test)] mod tests_and;
#[cfg(test)] mod tests_asl;
#[cfg(test)] mod tests_bit;
#[cfg(test)] mod tests_brk;
#[cfg(test)] mod tests_cmp;
#[cfg(test)] mod tests_cpx;
#[cfg(test)] mod tests_cpy;
#[cfg(test)] mod tests_dec;
#[cfg(test)] mod tests_dex;
#[cfg(test)] mod tests_dey;
#[cfg(test)] mod tests_eor;
#[cfg(test)] mod tests_inc;
#[cfg(test)] mod tests_inx;
#[cfg(test)] mod tests_iny;
#[cfg(test)] mod tests_ora;
#[cfg(test)] mod tests_sbc;
#[cfg(test)] mod tests_trb;
#[cfg(test)] mod tests_tsb;