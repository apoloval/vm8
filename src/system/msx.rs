use std::{io as stdio};
use std::time::Duration;

use crate::cpu::z80;
use crate::emu::{Freq, Scheduler};
use crate::io;

pub mod slot;

const IOPORT_PPI_A: z80::IOAddr = 0xa8;
const IOPORT_PPI_B: z80::IOAddr = 0xa9;
const IOPORT_PPI_C: z80::IOAddr = 0xaa;

const CPU_FREQ: Freq = Freq::from_khz(35_800);

pub struct MSX<S: slot::Config> {
  cpu: z80::CPU,
  slots: S,
  ppi: io::I8255,
}

impl<S: slot::Config> MSX<S> {
  pub fn new(slots: S) -> MSX<S> {
    let cpu = z80::CPU::new();
    let ppi = io::I8255::new();
    MSX { cpu, slots, ppi }
  }

  pub fn run(&mut self, scheduler: &mut Scheduler<Self>) {
    scheduler.add_task(|msx: &mut MSX<S>, dur: Duration| {
      let ppi = &mut msx.ppi;
      let slots = &mut msx.slots;
      let mut bus = Bus { ppi, slots };
      msx.cpu.exec(&mut bus, CPU_FREQ.cycles_in(dur));
    });
    scheduler.run(self);
  }
}

// Spectravideo 728 MSX system
pub type SVI728 = MSX<(
  slot::ROM32,
  slot::RAM64,
  slot::NotConnected,
  slot::NotConnected,
)>;

pub fn svi_728<R: stdio::Read>(rom_img: &mut R) -> stdio::Result<SVI728> {
  slot::ROM32::new(rom_img).map(|rom| MSX::new((
    rom,
    slot::RAM64::new(),
    slot::NotConnected,
    slot::NotConnected
  )))
}


pub struct Bus<'a, S: 'a + slot::Config> {
  ppi: &'a mut io::I8255,
  slots: &'a mut S,
}

impl<'a, S: 'a + slot::Config> Bus<'a, S> {
  fn slot(&self, addr: z80::MemAddr) -> u8 {
    let page = (addr.0 & 0xc000) >> 14;
    let cfg = self.ppi.port_a();
    match page {
      0 => cfg & 0x03,
      1 => (cfg >> 2) & 0x03,
      2 => (cfg >> 4) & 0x03,
      3 => (cfg >> 6) & 0x03,
      _ => unreachable!(),
    }
  }
}

impl<'a, S: 'a + slot::Config> z80::MemBus for Bus<'a, S> {
  fn mem_read(&self, addr: z80::MemAddr) -> u8 {
    match self.slot(addr) {
      0 => self.slots.slot0().mem_read(addr),
      1 => self.slots.slot1().mem_read(addr),
      2 => self.slots.slot2().mem_read(addr),
      3 => self.slots.slot3().mem_read(addr),
      _ => unreachable!(),
    }
  }

  fn mem_write(&mut self, addr: z80::MemAddr, val: u8) {
    match self.slot(addr) {
      0 => self.slots.slot0_mut().mem_write(addr, val),
      1 => self.slots.slot1_mut().mem_write(addr, val),
      2 => self.slots.slot2_mut().mem_write(addr, val),
      3 => self.slots.slot3_mut().mem_write(addr, val),
      _ => unreachable!(),
    }
  }
}

impl<'a, S: 'a + slot::Config> z80::IOBus for Bus<'a, S> {
  fn io_read(&self, addr: z80::IOAddr) -> u8 {
    match addr {
      IOPORT_PPI_A => self.ppi.port_a(),
      IOPORT_PPI_B => self.ppi.port_b(),
      IOPORT_PPI_C => self.ppi.port_c(),
      _ => 0xff,
    }
  }

  fn io_write(&mut self, addr: z80::IOAddr, val: u8) {
    match addr {
      IOPORT_PPI_A => self.ppi.set_port_a(val),
      IOPORT_PPI_B => self.ppi.set_port_b(val),
      IOPORT_PPI_C => self.ppi.set_port_c(val),
      _ => {},
    };
  }
}