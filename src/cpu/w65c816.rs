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

    fn direct_byte(&mut self, bus: &impl Bus, dir: u8, idx: u16) -> u8 {
        self.direct_word(bus, dir, idx) as u8
    }

    fn direct_word(&mut self, bus: &impl Bus, dir: u8, idx: u16) -> u16 {
        let wrap = if self.regs.mode_is_emulated() && self.regs.dl() == 0 {
            AddrWrap::Byte
        } else {
            AddrWrap::Word
        };
        let addr = Addr::from(0, self.regs.dp())
            .wrapping_add(dir, wrap)
            .wrapping_add(idx as u8, wrap);
        bus.read_word(addr, wrap)
}

    fn direct_ptr(&mut self, bus: &impl Bus, dir: u8, idx: u16) -> Addr {
        Addr::from(
            self.regs.dbr(), 
            self.direct_word(bus, dir, idx)
        )
    }

    fn direct_ptr_long(&mut self, bus: &impl Bus, dir: u8, idx: u16) -> Addr {    
        Addr::from(
            self.direct_byte(bus, dir, idx.wrapping_add(2)),
            self.direct_word(bus, dir, idx)
        )
    }

    fn stack_word(&mut self, bus: &impl Bus, idx: u16) -> u16 {
        let wrap = if self.regs.mode_is_emulated() { AddrWrap::Byte } 
        else { AddrWrap::Word };

        let addr = Addr::from(0, self.regs.sp())
            .wrapping_add(idx, wrap);
        bus.read_word(addr, wrap)
    }

    fn fetch_pc_byte<B: Bus>(&mut self, bus: &mut B, offset: u16) -> u8 {
        let addr = Addr::from(self.regs.pbr(), self.regs.pc())
            .wrapping_add(offset, AddrWrap::Word);
        bus.read_byte(addr)
    }

    fn fetch_pc_word<B: Bus>(&mut self, bus: &mut B, offset: u16) -> u16 {
        let addr = Addr::from(self.regs.pbr(), self.regs.pc())
            .wrapping_add(offset, AddrWrap::Word);
        bus.read_word(addr, AddrWrap::Word)
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
            0x05 => {
                // ORA d
                let dir = self.fetch_pc_byte(bus, 1);
                self.ora(bus, addr::Mode::Direct(dir), rep)
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
            0x0D => {
                // ORA a
                let abs = self.fetch_pc_word(bus, 1);
                self.ora(bus, addr::Mode::Absolute(abs), rep)
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
            0x15 => {
                // ORA d,X
                let dir = self.fetch_pc_byte(bus, 1);
                self.ora(bus, addr::Mode::DirectIndexedX(dir), rep)
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
            0x1D => {
                // ORA a,X
                let abs = self.fetch_pc_word(bus, 1);
                self.ora(bus, addr::Mode::AbsoluteIndexedX(abs), rep)
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

        let mode_eval = mode.eval(self, bus);
        let prev = self.regs.a();

        let carry = if self.regs.status_flag_is_set(Flag::C) { 1 } else { 0 };
        let result = if self.regs.status_flag_is_set(Flag::D) {
            bcd::add_word(
                bcd::add_word(prev, mode_eval.val),
                carry,
            )
        } else {
            prev.wrapping_add(mode_eval.val).wrapping_add(carry)
        };

        if self.regs.accum_is_byte() {
            self.regs.al_set(result as u8);
            self.regs.set_status_flag(Flag::N, result & 0x80 != 0);
            self.regs.set_status_flag(Flag::V, (result as i8) < (prev as i8));
            self.regs.set_status_flag(Flag::Z, result & 0x00FF == 0);
            self.regs.set_status_flag(Flag::C, (result as u8) < (prev as u8));
        } else {
            self.regs.a_set(result);
            self.regs.set_status_flag(Flag::N, result & 0x8000 != 0);
            self.regs.set_status_flag(Flag::V, (result as i16) < (prev as i16));
            self.regs.set_status_flag(Flag::Z, result == 0);
            self.regs.set_status_flag(Flag::C, (result as u16) < (prev as u16));
        }

        self.regs.pc_inc(mode_eval.bytes);
        self.cycles += mode_eval.cycles;
    }

    fn and(&mut self, bus: &mut impl Bus, mode: addr::Mode, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("AND"), 
            operands: format!("{}", mode),
        });

        let mode_eval = mode.eval(self, bus);
        let result = self.regs.a() & mode_eval.val;
        
        if self.regs.accum_is_byte() {
            self.regs.al_set(result as u8);
            self.regs.set_status_flag(Flag::Z, result & 0x00FF == 0);
            self.regs.set_status_flag(Flag::N, result & 0x80 != 0);
        } else {
            self.regs.a_set(result);
            self.regs.set_status_flag(Flag::Z, result == 0);
            self.regs.set_status_flag(Flag::N, result & 0x8000 != 0);
        
        }
        self.regs.pc_inc(mode_eval.bytes);
        self.cycles += mode_eval.cycles;
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

    fn eor(&mut self, bus: &mut impl Bus, mode: addr::Mode, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("EOR"), 
            operands: format!("{}", mode),
        });

        let mode_eval = mode.eval(self, bus);
        let result = self.regs.a() ^ mode_eval.val;
        
        if self.regs.accum_is_byte() {
            self.regs.al_set(result as u8);
            self.regs.set_status_flag(Flag::Z, result & 0x00FF == 0);
            self.regs.set_status_flag(Flag::N, result & 0x80 != 0);
        } else {
            self.regs.a_set(result);
            self.regs.set_status_flag(Flag::Z, result == 0);
            self.regs.set_status_flag(Flag::N, result & 0x8000 != 0);
        
        }
        self.regs.pc_inc(mode_eval.bytes);
        self.cycles += mode_eval.cycles;

    }

    fn ora(&mut self, bus: &mut impl Bus, mode: addr::Mode, rep: &mut impl Reporter) {
        rep.report(|| Event::Exec { 
            pbr: self.regs.pbr(),
            pc: self.regs.pc(),
            instruction: String::from("ORA"), 
            operands: format!("{}", mode),
        });

        let mode_eval = mode.eval(self, bus);
        let result = self.regs.a() | mode_eval.val;
        
        if self.regs.accum_is_byte() {
            self.regs.al_set(result as u8);
            self.regs.set_status_flag(Flag::Z, result & 0x00FF == 0);
            self.regs.set_status_flag(Flag::N, result & 0x80 != 0);
        } else {
            self.regs.a_set(result);
            self.regs.set_status_flag(Flag::Z, result == 0);
            self.regs.set_status_flag(Flag::N, result & 0x8000 != 0);
        
        }
        self.regs.pc_inc(mode_eval.bytes);
        self.cycles += mode_eval.cycles;

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
#[cfg(test)] mod tests_brk;
#[cfg(test)] mod tests_eor;
#[cfg(test)] mod tests_ora;
