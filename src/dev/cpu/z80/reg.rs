use byteorder::ByteOrder;

const REG_A: usize = 0;
const REG_F: usize = 1;
const REG_B: usize = 2;
const REG_C: usize = 3;
const REG_D: usize = 4;
const REG_E: usize = 5;
const REG_H: usize = 6;
const REG_L: usize = 7;
const REG_AF: usize = 0;
const REG_BC: usize = 2;
const REG_DE: usize = 4;
const REG_HL: usize = 6;
const REG_PC: usize = 8;
const REG_SP: usize = 10;
const REG_MAX: usize = 12;

#[derive(Default)]
pub struct Bank {
  main: [u8; REG_MAX],
}

impl Bank {
  pub fn a(&self) -> u8 { self.main[REG_A] }
  pub fn f(&self) -> u8 { self.main[REG_F] }
  pub fn b(&self) -> u8 { self.main[REG_B] }
  pub fn c(&self) -> u8 { self.main[REG_C] }
  pub fn d(&self) -> u8 { self.main[REG_D] }
  pub fn e(&self) -> u8 { self.main[REG_E] }
  pub fn h(&self) -> u8 { self.main[REG_H] }
  pub fn l(&self) -> u8 { self.main[REG_L] }

  pub fn af(&self) -> u16 { byteorder::BigEndian::read_u16(&self.main[REG_AF..]) }
  pub fn bc(&self) -> u16 { byteorder::BigEndian::read_u16(&self.main[REG_BC..]) }
  pub fn de(&self) -> u16 { byteorder::BigEndian::read_u16(&self.main[REG_DE..]) }
  pub fn hl(&self) -> u16 { byteorder::BigEndian::read_u16(&self.main[REG_HL..]) }
  pub fn pc(&self) -> u16 { byteorder::BigEndian::read_u16(&self.main[REG_PC..]) }
  pub fn sp(&self) -> u16 { byteorder::BigEndian::read_u16(&self.main[REG_SP..]) }

  pub fn set_a(&mut self, v: u8) { self.main[REG_A] = v; }
  pub fn set_f(&mut self, v: u8) { self.main[REG_F] = v; }
  pub fn set_b(&mut self, v: u8) { self.main[REG_B] = v; }
  pub fn set_c(&mut self, v: u8) { self.main[REG_C] = v; }
  pub fn set_d(&mut self, v: u8) { self.main[REG_D] = v; }
  pub fn set_e(&mut self, v: u8) { self.main[REG_E] = v; }
  pub fn set_h(&mut self, v: u8) { self.main[REG_H] = v; }
  pub fn set_l(&mut self, v: u8) { self.main[REG_L] = v; }

  pub fn set_af(&mut self, v: u16) { byteorder::BigEndian::write_u16(&mut self.main[REG_AF..], v); }
  pub fn set_bc(&mut self, v: u16) { byteorder::BigEndian::write_u16(&mut self.main[REG_BC..], v); }
  pub fn set_de(&mut self, v: u16) { byteorder::BigEndian::write_u16(&mut self.main[REG_DE..], v); }
  pub fn set_hl(&mut self, v: u16) { byteorder::BigEndian::write_u16(&mut self.main[REG_HL..], v); }
  pub fn set_pc(&mut self, v: u16) { byteorder::BigEndian::write_u16(&mut self.main[REG_PC..], v); }
  pub fn set_sp(&mut self, v: u16) { byteorder::BigEndian::write_u16(&mut self.main[REG_SP..], v); }
}
