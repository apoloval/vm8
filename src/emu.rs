use std::ops::{Add, Mul};
use std::time::{Instant, Duration};
use std::thread::sleep;

// An emulation task operating on a context of type C.
pub trait Task<C> {
  fn exec(&mut self, ctx: &mut C, dur: Duration);
}

impl<F: FnMut(&mut C, Duration), C> Task<C> for F {
  fn exec(&mut self, ctx: &mut C, dur: Duration) {
    (self)(ctx, dur);
  }
}

pub struct Scheduler<C> {
  tasks: Vec<Box<dyn Task<C>>>,
  slot_dur: Duration,
}

impl<C> Scheduler<C> {
  pub fn new() -> Scheduler<C> {
    let tasks = Vec::with_capacity(64);
    let slot_dur = Duration::from_millis(10);
    Scheduler { tasks, slot_dur }
  }

  pub fn add_task<T: Task<C> + 'static>(&mut self, task: T) {
    self.tasks.push(Box::new(task));
  }

  pub fn run(&mut self, ctx: &mut C) {
    loop {
      let mut elapsed = Duration::from_millis(0);
      for task in &mut self.tasks {
        let t0 = Instant::now();
        task.exec(ctx, self.slot_dur);
        elapsed += t0.elapsed();
      }
      if elapsed < self.slot_dur {
        sleep(self.slot_dur - elapsed);
      }
    }
  }
}

// A count of clock cycles
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cycles(pub u64);

impl Add for Cycles {
  type Output = Cycles;
  fn add(self, other: Cycles) -> Cycles {
    Cycles(self.0 + other.0)
  }
}

// Frequency in Hertzs
#[derive(Clone, Copy, Debug)]
pub struct Freq(u64);

impl Freq {
  pub const fn new(hz: u64) -> Freq {
    Freq(hz)
  }

  pub const fn from_khz(v: u64) -> Freq { Freq::new(1_000 * v) }
  pub const fn from_mhz(v: u64) -> Freq { Freq::new(1_000_000 * v) }
  pub const fn from_ghz(v: u64) -> Freq { Freq::new(1_000_000_000 * v) }

  pub fn cycles_in(&self, dur: Duration) -> Cycles {
    let one_sec = Duration::from_secs(1);
    Cycles(((dur.as_nanos() * self.0 as u128) / one_sec.as_nanos()) as u64)
  }
}

impl Mul<u64> for Freq {
  type Output = Freq;
  fn mul(self, other: u64) -> Freq {
    Freq(self.0 * other)
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn freq_cycles_in() {
    assert_eq!(
      Cycles(40_000),
      Freq::from_mhz(4).cycles_in(Duration::from_millis(10)),
    );
    assert_eq!(
      Cycles(35_800),
      Freq::from_khz(3580).cycles_in(Duration::from_millis(10)),
    );
  }
}