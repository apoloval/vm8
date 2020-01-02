use std::collections::HashMap;
use std::num::Wrapping;

use byteorder::{ByteOrder, LittleEndian};

mod exec;
mod inst;
mod regs;

use regs::RegBank;

type Cycles = usize;

type MemAddr = u16;
type IOAddr = u8;

// A memory bus
pub trait MemBus {
  fn mem_read(&self, addr: MemAddr) -> u8;
  fn mem_write(&mut self, addr: MemAddr, val: u8);

  fn mem_read16(&self, addr: MemAddr) -> u16 {
    let Wrapping(addr2) = Wrapping(addr) + Wrapping(1);
    let bytes = [ self.mem_read(addr), self.mem_read(addr2) ];
    LittleEndian::read_u16(&bytes)
  }

  fn mem_write16(&mut self, addr: MemAddr, val: u16) {
    let Wrapping(addr2) = Wrapping(addr) + Wrapping(1);
    let mut data = [0; 2];
    LittleEndian::write_u16(&mut data, val);
    self.mem_write(addr, data[0]);
    self.mem_write(addr2, data[1]);
  }
}

// An implementation of a memory bus for vectors for testing purposes
impl MemBus for Vec<u8> {
  fn mem_read(&self, addr: MemAddr) -> u8 {
    let offset = (addr as usize) % self.len();
    self[offset]
  }

  fn mem_write(&mut self, addr: MemAddr, val: u8) {
    let offset = (addr as usize) % self.len();
    self[offset] = val;
  }
}

// An implementation of memory bus for any binary tuple having a mem bus as first element.
// This is useful for testing, since it enables a tuple of mem and IO buses to implement `Bus`.
impl<T1: MemBus, T2> MemBus for (T1, T2) {
  fn mem_read(&self, addr: MemAddr) -> u8 { self.0.mem_read(addr) }
  fn mem_write(&mut self, addr: MemAddr, val: u8) { self.0.mem_write(addr, val) }
}

// An IO bus
pub trait IOBus {
  fn io_read(&self, addr: IOAddr) -> u8;
  fn io_write(&mut self, addr: IOAddr, val: u8);
}

// An implementation of IO bus for hashmaps for testing purposes
impl IOBus for HashMap<IOAddr, u8> {
  fn io_read(&self, addr: IOAddr) -> u8 { self[&addr] }
  fn io_write(&mut self, addr: IOAddr, val: u8) { self.insert(addr, val); }
}

// An implementation of IO bus for any binary tuple having an IO bus as second element.
// This is useful for testing, since it enables a tuple of mem and IO buses to implement `Bus`.
impl<T1, T2: IOBus> IOBus for (T1, T2) {
  fn io_read(&self, addr: IOAddr) -> u8 { self.1.io_read(addr) }
  fn io_write(&mut self, addr: IOAddr, val: u8) { self.1.io_write(addr, val) }
}

pub trait Bus : MemBus + IOBus {}

impl<T: MemBus + IOBus> Bus for T {}

pub struct CPU<B: Bus> {
  bus: B,
  regs: RegBank,
}

impl<B: Bus> CPU<B> {
  pub fn new(bus: B) -> CPU<B> {
    CPU {
      regs: RegBank::default(),
      bus,
    }
  }
}

impl CPU<(Vec<u8>, HashMap<IOAddr, u8>)> {
  fn testbench() -> CPU<(Vec<u8>, HashMap<IOAddr, u8>)> {
    let mut memory = Vec::with_capacity(64*1024);
    memory.resize(64*1024, 0);
    let io = HashMap::new();
    let bus = (memory, io);
    Self::new(bus)
  }
}

impl<B: Bus> exec::Context for CPU<B> {
  type Mem = B;
  type IO = B;

  fn regs(&self) -> &RegBank { &self.regs }
  fn regs_mut(&mut self) -> &mut RegBank { &mut self.regs }
  fn mem(&self) -> &Self::Mem { &self.bus }
  fn mem_mut(&mut self) -> &mut Self::Mem { &mut self.bus }
  fn io(&self) -> &Self::IO { &self.bus }
  fn io_mut(&mut self) -> &mut Self::IO { &mut self.bus }
}
