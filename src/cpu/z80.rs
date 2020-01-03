use std::collections::HashMap;
use std::num::Wrapping;

use byteorder::{ByteOrder, LittleEndian};

use crate::emu::Cycles;

mod exec;
mod inst;
mod regs;

use regs::RegBank;

pub type MemAddr = u16;
pub type IOAddr = u8;


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


// A Z80 bus, comprised by memory and IO sub-buses.
pub trait Bus : MemBus + IOBus {}

impl<T: MemBus + IOBus> Bus for T {}


// The Z80 CPU.
pub struct CPU {
  regs: RegBank,
}

impl CPU {
  pub fn new() -> CPU {
    CPU {
      regs: RegBank::default(),
    }
  }

  pub fn exec<B: Bus>(&mut self, bus: &mut B, max_cycles: Cycles) {
    let mut ctx = CPUContext {
      regs: &mut self.regs,
      bus: bus,
    };
    let mut total_cycles = Cycles(0);
    while total_cycles < max_cycles {
      total_cycles = total_cycles + inst::exec_inst(&mut ctx);
    }
  }
}

// A value that contains the context of the CPU for its execution.
struct CPUContext<'a, B: 'a + Bus> {
  regs: &'a mut RegBank,
  bus: &'a mut B,
}

impl<'a, B: 'a + Bus> exec::Context for CPUContext<'a, B> {
  type Mem = B;
  type IO = B;

  fn regs(&self) -> &RegBank { &self.regs }
  fn regs_mut(&mut self) -> &mut RegBank { &mut self.regs }
  fn mem(&self) -> &Self::Mem { &self.bus }
  fn mem_mut(&mut self) -> &mut Self::Mem { &mut self.bus }
  fn io(&self) -> &Self::IO { &self.bus }
  fn io_mut(&mut self) -> &mut Self::IO { &mut self.bus }
}


// A testbench for Z80 testing. This type implements `exec::Context` trait with trivial
// memory and IO registers, what makes it a good candidate for testing purposes.
pub struct TestBench {
  pub regs: RegBank,
  pub mem: Vec<u8>,
  pub io: HashMap<IOAddr, u8>,
}

impl TestBench {
  pub fn new() -> TestBench {
    let regs = RegBank::default();
    let mut mem = Vec::with_capacity(64*1024);
    mem.resize(64*1024, 0);
    let io = HashMap::new();
    TestBench { regs, mem, io }
  }
}

impl exec::Context for TestBench {
  type Mem = Vec<u8>;
  type IO = HashMap<IOAddr, u8>;

  fn regs(&self) -> &RegBank { &self.regs }
  fn regs_mut(&mut self) -> &mut RegBank { &mut self.regs }
  fn mem(&self) -> &Self::Mem { &self.mem }
  fn mem_mut(&mut self) -> &mut Self::Mem { &mut self.mem }
  fn io(&self) -> &Self::IO { &self.io }
  fn io_mut(&mut self) -> &mut Self::IO { &mut self.io }
}
