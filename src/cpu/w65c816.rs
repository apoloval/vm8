mod bus;
mod ev;
mod int;
mod addr;
mod reg;
mod status;
#[cfg(test)] mod assert;

pub use bus::{Addr, AddrWrap, Bus};
pub use ev::{Event, Reporter, NullReporter};

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
            _ => unimplemented!()
        }
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
                    cpu.regs.set_status_flag(Flag::O, val == "1"),
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

#[cfg(test)]
mod tests {
    use self::status::FlagExpectation;

    use super::*;
    use crate::cpu::w65c816::assert;

    use rstest::*;
    
    #[rstest]
    #[case::emulation(
        "P.E:1,PC:A000,P:AA,SP:FF",             // cpu
        "00A000:0000,00FFFE:3412",              // bus
        vec![0xBA, 0x02, 0xA0],                 // expected_stack
        0x1234,                                 // expected_pc        
        0xBA,                                   // expected_state
    )]
    #[case::native(
        "P.E:0,PBR:B0,PC:A000,P:AA,SP:E0FF",    // cpu
        "B0A000:0000,00FFE6:3412",              // bus
        vec![0xAA, 0x02, 0xA0, 0xB0],           // expected_stack
        0x1234,                                 // expected_pc        
        0xAA,                                   // expected_state
    )]
    fn test_brk(
        #[case] mut cpu: CPU, 
        #[case] mut bus: bus::Fake,
        #[case] expected_stack: Vec<u8>,
        #[case] expected_pc: u16,
        #[case] expected_state: u8,
    ) {
        cpu.step(&mut bus, &mut NullReporter);

        for (offset, expected) in expected_stack.iter().enumerate() {
            assert::stack_byte(&cpu, &bus, offset as u16+1, *expected);
        }
        assert::program_counter(&cpu, 0, expected_pc);
        assert::program_state(&cpu, expected_state);
    }

    #[rstest]
    #[case::emulation(
        "P.E:1,PC:A000,A:1160",                         // cpu
        "00A000:0903",                                  // bus
        0x1163,                                         // expected
        "Z:0,N:0",                                      // expected_flags_set
    )]
    #[case::native_8bit(
        "P.E:0,P.M:1,PC:A000,A:1160",                   // cpu
        "00A000:0903",                                  // bus
        0x1163,                                         // expected
        "Z:0,N:0",                                      // expected_flags_set
    )]
    #[case::native_16bit(
        "P.E:0,P.M:0,PC:A000,A:1160",                   // cpu
        "00A000:090302",                                // bus
        0x1363,                                         // expected
        "Z:0,N:0",                                      // expected_flags_set
    )]
    #[case::native_8bit_zero(
        "P.E:0,P.M:1,PC:A000,A:1100",                   // cpu
        "00A000:0900",                                  // bus
        0x1100,                                         // expected
        "Z:1,N:0",                                      // expected_flags_set
    )]
    #[case::native_16bit_zero(
        "P.E:0,P.M:0,PC:A000,A:0000",                   // cpu
        "00A000:090000",                                // bus
        0x0000,                                         // expected
        "Z:1,N:0",                                      // expected_flags_set
    )]
    #[case::native_8bit_neg(
        "P.E:0,P.M:1,PC:A000,A:1180",                   // cpu
        "00A000:0908",                                  // bus
        0x1188,                                         // expected
        "Z:0,N:1",                                      // expected_flags_set
    )]
    #[case::native_16bit_neg(
        "P.E:0,P.M:0,PC:A000,A:8080",                   // cpu
        "00A000:090808",                                // bus
        0x8888,                                         // expected
        "Z:0,N:1",                                      // expected_flags_set
    )]
    fn test_ora_results(
        #[case] mut cpu: CPU,
        #[case] mut bus: bus::Fake,
        #[case] expected: u16,
        #[case] expected_flags: FlagExpectation,
    ) {
        cpu.step(&mut bus, &mut NullReporter);

        assert::accum(&cpu, expected);
        expected_flags.assert(cpu.regs.p());
    }


    /* 
     * Test ORA instruction decoding. The test cases just have to prepare the CPU and the bus
     * with the program code and the memory state to run the ORA instruction with a specific
     * addressing mode. The test assumes A is fully reset before the instruction is executed, 
     * and it will just check A is not zero after the instruction is executed.
     */
    #[rstest]
    #[case::absolute(
        "PC:A000",                                      // cpu
        "00A000:0D5634",                                // bus
        ("ORA", "$3456"),                               // expected
        0xA003,                                         // expected_pc
    )]
    #[case::absolute_indexed_x(
        "PC:A000",                                      // cpu
        "00A000:1D5634",                                // bus
        ("ORA", "$3456,X"),                             // expected
        0xA003,                                         // expected_pc
    )]
    #[case::absolute_indexed_y(
        "PC:A000",                                      // cpu
        "00A000:195634",                                // bus
        ("ORA", "$3456,Y"),                             // expected
        0xA003,                                         // expected_pc
    )]
    #[case::absolute_long(
        "PC:A000",                                      // cpu
        "00A000:0F563412",                              // bus
        ("ORA", "$123456"),                             // expected
        0xA004,                                         // expected_pc
    )]
    #[case::absolute_long_indexed(
        "PC:A000",                                      // cpu
        "00A000:1F563412",                              // bus
        ("ORA", "$123456,X"),                           // expected
        0xA004,                                         // expected_pc
    )]
    #[case::direct(
        "PC:A000",                                      // cpu
        "00A000:0504",                                  // bus
        ("ORA", "$04"),                                 // expected
        0xA002,                                         // expected_pc
    )]
    #[case::direct_indirect_indexed(
        "PC:A000",                                      // cpu
        "00A000:1104",                                  // bus
        ("ORA", "($04),Y"),                             // expected
        0xA002,                                         // expected_pc
    )]
    #[case::direct_indexed_indirect(
        "PC:A000",                                      // cpu
        "00A000:0104",                                  // bus
        ("ORA", "($04,X)"),                             // expected
        0xA002,                                         // expected_pc
    )]
    #[case::direct_indexed_x(
        "PC:A000",                                      // cpu
        "00A000:1504",                                  // bus
        ("ORA", "$04,X"),                               // expected
        0xA002,                                         // expected_pc
    )]
    #[case::direct_indirect(
        "PC:A000",                                      // cpu
        "00A000:1244",                                  // bus
        ("ORA", "($44)"),                               // expected
        0xA002,                                         // expected_pc
    )]
    #[case::direct_indirect_long(
        "PC:A000",                                      // cpu
        "00A000:0744",                                  // bus
        ("ORA", "[$44]"),                               // expected
        0xA002,                                         // expected_pc
    )]
    #[case::direct_indirect_long_indexed(
        "PC:A000",                                      // cpu
        "00A000:1744",                                  // bus
        ("ORA", "[$44],Y"),                             // expected
        0xA002,                                         // expected_pc
    )]
    #[case::immediate(
        "PC:A000",                                      // cpu
        "00A000:09FFFF",                                // bus
        ("ORA", "#$FFFF"),                              // expected
        0xA003,                                         // expected_pc
    )]
    #[case::stack_relative(
        "PC:A000",                                      // cpu
        "00A000:0304",                                  // bus
        ("ORA", "$04,S"),                               // expected
        0xA002,                                         // expected_pc
    )]
    #[case::stack_relative_indirect_indexed(
        "PC:A000",                                      // cpu
        "00A000:1304",                                  // bus
        ("ORA", "($04,S),Y"),                           // expected
        0xA002,                                         // expected_pc
    )]
    fn test_ora_decoding(
        #[case] mut cpu: CPU,
        #[case] mut bus: bus::Fake,
        #[case] expected: (&'static str, &'static str),
        #[case] expected_pc: u16,
    ) {
        let mut reporter = ev::Retain::new();
        cpu.step(&mut bus, &mut reporter);

        let (expected_inst, expected_ops) = expected;
        reporter.assert_exec(expected_inst, expected_ops);
        assert::program_counter(&cpu, cpu.regs.pbr(), expected_pc);
    }
}