use super::{int, status, Addr, AddrWrap, Bus};

#[derive(Default)]
pub struct Bank {
    p: u8,      // Processor Status Register
    e: bool,    // Emulation Mode flag

    a: u16,     // Accumulator
    x: u16,     // Index Register X
    y: u16,     // Index Register Y

    pc: u16,    // Program Counter
    sp: u16,    // Stack Pointer
    dp: u16,    // Direct Page Register
    pbr: u8,    // Program Bank Register
    dbr: u8,    // Data Bank Register
}

#[allow(dead_code)]
impl Bank {
    pub fn reset<B: Bus>(&mut self, bus: &mut B) {
        self.e = true;
        self.pc = bus.read_word(
            Addr::from(0, int::VECTOR_EMULATION_RESET),
            AddrWrap::Long,
        );
        self.pbr = 0;
        self.dbr = 0;
        self.dp = 0;
        self.set_mode_emulated();
    }

    pub fn status_flag_is_set(&self, flag: status::Flag) -> bool { self.p & flag.mask() != 0 }
    pub fn status_flag_is_clear(&self, flag: status::Flag) -> bool { self.p & flag.mask() == 0 }

    pub fn set_status_flag(&mut self, flag: status::Flag, active: bool) { 
        if active { flag.set(&mut self.p); } 
        else { flag.clear(&mut self.p); }

        if self.mode_is_emulated() {
            self.p |= status::Flag::M.mask() | status::Flag::X.mask();
        }
    }

    pub fn set_mode_emulated(&mut self) {
        self.e = true;

        // When the e flag is 1, the SH register is forced to $01...
        self.force_sp_emulation();


        // ...the m flag is forced to 1, and the x flag is forced to 1. 
        self.p |= status::Flag::M.mask() | status::Flag::X.mask();

        // As a consequence of the x flag being forced to 1, the XH register and the YH register
        // are forced to $00.
        self.x &= 0x00FF;
        self.y &= 0x00FF;
    }

    pub fn set_mode_native(&mut self) {
        self.e = false;
    }

    pub fn mode_is_emulated(&self) -> bool { self.e }
    pub fn mode_is_native(&self) -> bool { !self.e }

    pub fn accum_is_byte(&self) -> bool { self.p & status::Flag::M.mask() != 0 }
    pub fn index_is_byte(&self) -> bool { self.p & status::Flag::X.mask() != 0 }

    pub fn p(&self) -> u8 { self.p }
    pub fn p_set(&mut self, value: u8) { self.p = value; }

    pub fn dp(&self) -> u16 { self.dp }
    pub fn dl(&self) -> u8 { self.dp as u8 }
    pub fn dh(&self) -> u8 { (self.dp >> 8) as u8 }

    pub fn dp_set(&mut self, value: u16) { self.dp = value; }

    pub fn a(&self) -> u16 { self.a }
    pub fn x(&self) -> u16 { self.x }
    pub fn y(&self) -> u16 { self.y }

    pub fn al(&self) -> u8 { self.a as u8 }
    pub fn al_set(&mut self, value: u8) { self.a = (self.a & 0xFF00) | (value as u16) }

    pub fn a_set(&mut self, value: u16) { self.a = value }

    pub fn x_set(&mut self, value: u16) { 
        self.x = value; 
        if self.index_is_byte() { self.x &= 0x00FF }
    }

    pub fn y_set(&mut self, value: u16) { 
        self.y = value; 
        if self.index_is_byte() { self.y &= 0x00FF }
    }

    pub fn pbr(&self) -> u8 { if self.mode_is_native() { self.pbr } else { 0 } }
    pub fn pbr_set(&mut self, value: u8) { self.pbr = value; }

    pub fn dbr(&self) -> u8 { if self.mode_is_native() { self.dbr } else { 0 } }
    pub fn dbr_set(&mut self, value: u8) { self.dbr = value; }

    pub fn sp(&self) -> u16 { self.sp }
    pub fn sp_inc(&mut self, n: u16) { self.sp_set(self.sp.wrapping_add(n)); }
    pub fn sp_dec(&mut self, n: u16) { self.sp_set(self.sp.wrapping_sub(n)); }    
    pub fn sp_set(&mut self, n: u16) { 
        self.sp = n; 
        if self.mode_is_emulated() { self.force_sp_emulation() }
    }

    pub fn pc(&self) -> u16 { self.pc }
    pub fn pc_inc(&mut self, n: u16) -> u16 { self.pc += n; self.pc }
    pub fn pc_jump(&mut self, n: u16) { self.pc = n; }

    #[inline]
    fn force_sp_emulation(&mut self) {
        self.sp &= 0x00FF;
        self.sp |= 0x0100;
    }
}
