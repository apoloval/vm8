use std::fmt;
use std::thread;
use std::time::{Duration, Instant};

pub type Cycles = usize;

#[derive(Clone, Copy, PartialEq)]
pub struct Frequency(f64);

impl Frequency {
    pub fn new(val: f64) -> Frequency {
        Frequency(val)
    }

    pub fn from_khz(val: f64) -> Frequency {
        Self::new(val * 1_000.0)
    }

    pub fn from_mhz(val: f64) -> Frequency {
        Self::new(val * 1_000_000.0)
    }

    pub fn period(&self) -> Duration {
        let secs = 1.0 / self.0;
        let nanos = secs * 1_000_000_000.0;
        Duration::new(secs as u64, nanos as u32)
    }
}

impl fmt::Display for Frequency {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if self.0 > 1_000_000.0 {
            write!(f, "{:.2}Mhz", self.0 / 1_000_000.0)
        } else {
            write!(f, "{:.2}Khz", self.0 / 1_000.0)
        }
    }
}

pub struct Clock { 
    cycle_period: Duration,   
    adjust_period: Cycles,
    cycles: Cycles,
    t0: Instant,
    native_freq: Option<Frequency>,
}

impl Clock {
    pub fn new(freq: Frequency) -> Clock {
        let cycle_period = freq.period();
        let adjust_period = 10_000_000 / cycle_period.subsec_nanos() as Cycles;
        Clock { 
            cycle_period: freq.period(), 
            adjust_period, 
            cycles: 0, 
            t0: Instant::now(),
            native_freq: None,
        }
    }

    pub fn native_freq(&self) -> Option<Frequency> { self.native_freq }

    pub fn walk(&mut self, cycles: Cycles) {
        self.cycles += cycles;

        if self.cycles >= self.adjust_period {
            self.refresh_native_freq();
            let actual = self.t0.elapsed();
            let expected = self.cycle_period * self.cycles as u32;
            if expected > actual {
                thread::sleep(expected - actual);
            } else {                
                panic!(
                    "emulated CPU is faster than physical ({}ns late per {} cycles, reached {})", 
                    (actual - expected).subsec_nanos(), 
                    self.cycles,
                    self.native_freq().unwrap(),
                );
            }
            self.reset();
        }
    }

    pub fn reset(&mut self) {
        self.cycles = 0;
        self.t0 = Instant::now();
    }

    fn refresh_native_freq(&mut self) {
        let elapsed = self.t0.elapsed();
        let elapsed_secs = elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
        self.native_freq = Some(Frequency::new(self.cycles as f64 / elapsed_secs));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn frequency_period() {
        let freq = Frequency::from_mhz(3.58);
        assert_eq!(Duration::from_nanos(279), freq.period());
    }

    #[test]
    fn frequency_display() {
        assert_eq!("256.29Khz", format!("{}", Frequency::from_khz(256.29)));
        assert_eq!("3.58Mhz", format!("{}", Frequency::from_mhz(3.58)));
    }
}