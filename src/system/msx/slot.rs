use std::io;

use crate::cpu::z80;

// A MSX slot basically as a Z80 memory bus
pub trait Slot: z80::MemBus {}

impl<T: z80::MemBus> Slot for T {}

pub trait Config {
  type Slot0: Slot;
  type Slot1: Slot;
  type Slot2: Slot;
  type Slot3: Slot;

  fn slot0(&self) -> &Self::Slot0;
  fn slot1(&self) -> &Self::Slot1;
  fn slot2(&self) -> &Self::Slot2;
  fn slot3(&self) -> &Self::Slot3;

  fn slot0_mut(&mut self) -> &mut Self::Slot0;
  fn slot1_mut(&mut self) -> &mut Self::Slot1;
  fn slot2_mut(&mut self) -> &mut Self::Slot2;
  fn slot3_mut(&mut self) -> &mut Self::Slot3;
}

impl<S0, S1, S2, S3> Config for (S0, S1, S2, S3)
where S0: z80::MemBus, S1: z80::MemBus, S2: z80::MemBus, S3: z80::MemBus {
  type Slot0 = S0;
  type Slot1 = S1;
  type Slot2 = S2;
  type Slot3 = S3;

  fn slot0(&self) -> &S0 { &self.0 }
  fn slot1(&self) -> &S1 { &self.1 }
  fn slot2(&self) -> &S2 { &self.2 }
  fn slot3(&self) -> &S3 { &self.3 }

  fn slot0_mut(&mut self) -> &mut S0 { &mut self.0 }
  fn slot1_mut(&mut self) -> &mut S1 { &mut self.1 }
  fn slot2_mut(&mut self) -> &mut S2 { &mut self.2 }
  fn slot3_mut(&mut self) -> &mut S3 { &mut self.3 }
}

// A slot that is not connected
pub struct NotConnected;

impl z80::MemBus for NotConnected {
  fn mem_read(&self, _addr: z80::MemAddr) -> u8 { 0xff }
  fn mem_write(&mut self, _addr: z80::MemAddr, _val: u8) {}
}

// A slot with 64KB of RAM memory
pub struct RAM64 {
  bytes: Box<[u8;64*1024]>,
}

impl RAM64 {
  pub fn new() -> RAM64 {
    let bytes = Box::new([0; 64*1024]);
    RAM64 { bytes }
  }
}

impl z80::MemBus for RAM64 {
  fn mem_read(&self, addr: z80::MemAddr) -> u8 {
    self.bytes[addr as usize]
  }

  fn mem_write(&mut self, addr: z80::MemAddr, val: u8) {
    self.bytes[addr as usize] = val;
  }
}

// A slot with 32KB of ROM memory
pub struct ROM32 {
  bytes: Box<[u8; 32*1024]>,
}

impl ROM32 {
  pub fn new<R: io::Read>(contents: &mut R) -> io::Result<ROM32> {
    let mut bytes = Box::new([0; 32*1024]);
    contents.read(&mut *bytes).map(|_| ROM32 { bytes })
  }

  pub fn fill(value: u8) -> ROM32 {
    let bytes = Box::new([value; 32*1024]);
    ROM32 { bytes }
  }
}

impl z80::MemBus for ROM32 {
  fn mem_read(&self, addr: z80::MemAddr) -> u8 {
    self.bytes[(addr & 0x7fff) as usize]
  }

  fn mem_write(&mut self, _addr: z80::MemAddr, _val: u8) {}
}

// A expanded slot
pub struct Expanded<C: Config> {
  subslots: C,
  pagecfg: u8,
}

impl<C: Config> Expanded<C> {
  pub fn new(subslots: C) -> Expanded<C> {
    let pagecfg = 0;
    Expanded { subslots, pagecfg }
  }

  fn slot(&self, addr: z80::MemAddr) -> Option<u8> {
    if addr != 0xffff {
      let page = (addr & 0xc000) >> 14;
      Some(match page {
        0 => self.pagecfg & 0x03,
        1 => (self.pagecfg >> 2) & 0x03,
        2 => (self.pagecfg >> 4) & 0x03,
        3 => (self.pagecfg >> 6) & 0x03,
        _ => unreachable!(),
      })
    } else { None }
  }
}

impl<C: Config> z80::MemBus for Expanded<C> {
  fn mem_read(&self, addr: z80::MemAddr) -> u8 {
    match self.slot(addr) {
      Some(0) => self.subslots.slot0().mem_read(addr),
      Some(1) => self.subslots.slot1().mem_read(addr),
      Some(2) => self.subslots.slot2().mem_read(addr),
      Some(3) => self.subslots.slot3().mem_read(addr),
      Some(_) => unreachable!(),
      None => !self.pagecfg,
    }
  }

  fn mem_write(&mut self, addr: z80::MemAddr, val: u8) {
    match self.slot(addr) {
      Some(0) => self.subslots.slot0_mut().mem_write(addr, val),
      Some(1) => self.subslots.slot1_mut().mem_write(addr, val),
      Some(2) => self.subslots.slot2_mut().mem_write(addr, val),
      Some(3) => self.subslots.slot3_mut().mem_write(addr, val),
      Some(_) => unreachable!(),
      None => self.pagecfg = val,
    }
  }
}

#[cfg(test)]
mod test {
  use crate::cpu::z80::MemBus;
  use super::*;

  #[test]
  fn ram64_read_write() {
    let mut ram = RAM64::new();
    for i in 0..64*1024 {
      ram.mem_write(i as u16, (i & 0xff) as u8);
    }

    assert_eq!(0x20, ram.mem_read(0x0020));
    assert_eq!(0x21, ram.mem_read(0x0121));
    assert_eq!(0x22, ram.mem_read(0x2022));
    assert_eq!(0x23, ram.mem_read(0xf023));
  }

  #[test]
  fn rom32_read() {
    let mut contents = vec![0; 32*1024];
    for i in 0..32*1024 {
      contents[i] = (i & 0xff) as u8;
    }
    let mut bytes: &[u8] = &mut contents[..];
    let rom = ROM32::new(&mut bytes).unwrap();

    assert_eq!(0x20, rom.mem_read(0x0020));
    assert_eq!(0x21, rom.mem_read(0x0121));
    assert_eq!(0x22, rom.mem_read(0x2022));
    assert_eq!(0x23, rom.mem_read(0xf023));
  }

  #[test]
  fn expanded_read_write() {
    let roms = (
      ROM32::fill(0), // slot 0 is filled with 0s
      ROM32::fill(1), // slot 1 is filled with 1s
      ROM32::fill(2), // slot 2 is filled with 2s
      ROM32::fill(3), // slot 3 is filled with 3s
    );
    let mut slot = Expanded::new(roms);

    // Initial config: all pages pointing to subslot 0
    assert_eq!(0xff, slot.mem_read(0xffff));
    assert_eq!(0, slot.mem_read(0x1001));
    assert_eq!(0, slot.mem_read(0x5001));
    assert_eq!(0, slot.mem_read(0x9001));
    assert_eq!(0, slot.mem_read(0xd001));

    // Let's configure page i in subslot i
    slot.mem_write(0xffff, 0b11100100);
    assert_eq!(0b00011011, slot.mem_read(0xffff));
    assert_eq!(0, slot.mem_read(0x1001));
    assert_eq!(1, slot.mem_read(0x5001));
    assert_eq!(2, slot.mem_read(0x9001));
    assert_eq!(3, slot.mem_read(0xd001));
  }
}