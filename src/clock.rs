use std::cmp::{Eq, Ord, Ordering};
use std::fmt;
use std::thread;
use std::time::Duration;

use time::PreciseTime;

pub type Cycles = usize;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
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

    pub fn to_mhz(&self) -> f64 {
        self.0 / 1_000_000.0
    }
}

impl fmt::Display for Frequency {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if self.0 > 1_000_000_000.0 {
            write!(f, "{:.2}Ghz", self.0 / 1_000_000_000.0)
        } else if self.0 > 1_000_000.0 {
            write!(f, "{:.2}Mhz", self.0 / 1_000_000.0)
        } else {
            write!(f, "{:.2}Khz", self.0 / 1_000.0)
        }
    }
}

impl Ord for Frequency {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 == other.0 {
            Ordering::Equal
        } else if self.0 < other.0 {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}


impl Eq for Frequency {}

pub struct FrequencyStats {
    pub avg: Frequency,
    pub p95: Frequency,
    pub p99: Frequency,
    pub max: Frequency,
    pub min: Frequency,
}

impl FrequencyStats {
    pub fn evaluate(mut data: Vec<Frequency>) -> Self {
        data.sort();

        // Calculate avg
        let sum = data.iter().fold(0f64, |acc, val| acc + val.to_mhz());
        let avg = Frequency::from_mhz(sum / data.len() as f64);

        // Calculate percentiles
        let p95 = data[(data.len() as f32 * 0.95) as usize];
        let p99 = data[(data.len() as f32 * 0.99) as usize];

        // Max & min
        let max = *data.last().unwrap();
        let min = *data.first().unwrap();

        Self { avg, p95, p99, max, min }
    }
}

pub struct SyncReport {
    pub real_duration: Duration,
    pub emulated_duration: Duration,
    pub native_freq: Frequency,
}

pub struct Clock {
    cycle_period: Duration,
    synced_at: PreciseTime,
}

impl Clock {
    pub fn new(freq: Frequency) -> Clock {
        Clock {
            cycle_period: freq.period(),
            synced_at: PreciseTime::now(),
        }
    }

    pub fn reset(&mut self) {
        self.synced_at = PreciseTime::now();
    }

    /// Synchronize the real time clock according to the frequency and the elapsed cycles.
    /// It returns a report with useful stats about the synchronization.
    pub fn sync(&mut self, cycles: Cycles, wait: bool) -> SyncReport {
        let actual_elapsed = self.synced_at.to(PreciseTime::now()).to_std().unwrap();
        let expected_elapsed = self.cycle_period * cycles as u32;
        let actual_freq = Frequency::from_elapsed(cycles, actual_elapsed);

        if wait {
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
        }
        self.synced_at = PreciseTime::now();
        SyncReport {
            real_duration: actual_elapsed,
            emulated_duration: expected_elapsed,
            native_freq: actual_freq,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn frequency_from_elapsed() {
        let freq = Frequency::from_elapsed(10_000_000, Duration::from_millis(10));
        assert_eq!(1000.0, freq.to_mhz());
    }

    #[test]
    fn frequency_period() {
        let freq = Frequency::from_mhz(3.58);
        assert_eq!(Duration::from_nanos(279), freq.period());
    }

    #[test]
    fn frequency_display() {
        assert_eq!("256.29Khz", format!("{}", Frequency::from_khz(256.29)));
        assert_eq!("3.58Mhz", format!("{}", Frequency::from_mhz(3.58)));
        assert_eq!("1.25Ghz", format!("{}", Frequency::from_mhz(1250.0)));
    }
}