// A (naive) implementation of Intel 8255 PPI.
pub struct I8255 {
  regs: [u8; 3],
}

impl I8255 {
  pub fn new() -> I8255 {
    let regs = [0; 3];
    I8255 { regs }
  }

  pub fn port_a(&self) -> u8 { self.regs[0] }
  pub fn port_b(&self) -> u8 { self.regs[1] }
  pub fn port_c(&self) -> u8 { self.regs[2] }

  pub fn set_port_a(&mut self, val: u8) { self.regs[0] = val }
  pub fn set_port_b(&mut self, val: u8) { self.regs[1] = val }
  pub fn set_port_c(&mut self, val: u8) { self.regs[2] = val }
}
