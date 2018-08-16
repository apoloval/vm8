use std::fmt;
use std::thread;
use std::time::{Duration, Instant};

pub type Cycles = usize;

#[derive(Clone, Copy, PartialEq)]
pub struct Frequency(f64);

impl Frequency {
    pub fn new(val: f64) -> Self {
        Frequency(val)
    }

    pub fn from_elapsed(cycles: usize, duration: Duration) -> Self {
        let elapsed_secs = duration.subsec_nanos() as f64 / 1_000_000_000.0;
        Self::new(cycles as f64 / elapsed_secs)
    }

    pub fn from_khz(val: f64) -> Self {
        Self::new(val * 1_000.0)
    }

    pub fn from_mhz(val: f64) -> Self {
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
    synced_at: Instant,
}

impl Clock {
    pub fn new(freq: Frequency) -> Clock {
        Clock { 
            cycle_period: freq.period(), 
            synced_at: Instant::now(),
        }
    }

    /// Synchronize the real time clock according to the frequency and the elapsed cycles.
    /// It returns the actual, native frequency inferred from the synchronization.
    pub fn sync(&mut self, cycles: Cycles) -> Frequency {
        let actual_elapsed = self.synced_at.elapsed();
        let expected_elapsed = self.cycle_period * cycles as u32;
        let actual_freq = Frequency::from_elapsed(cycles, actual_elapsed);

        if expected_elapsed > actual_elapsed {
            thread::sleep(expected_elapsed - actual_elapsed);
        } else {                
            panic!(
                "emulated clock is faster than physical ({}ns late per {} cycles, reached {})", 
                (actual_elapsed - expected_elapsed).subsec_nanos(), 
                cycles,
                actual_freq,
            );
        }
        self.synced_at = Instant::now();
        actual_freq
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