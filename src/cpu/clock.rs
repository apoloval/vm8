use std::thread;
use std::time::Duration;

pub type Cycles = usize;

pub struct Frequency(f64);

impl Frequency {
    pub fn new(val: f64) -> Frequency {
        Frequency(val)
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

pub struct Clock { 
    cycle_period: Duration,   
    adjust_period: Cycles,
    cycles: Cycles,
    elapsed: Duration,
}

impl Clock {
    pub fn new(freq: Frequency, adjust_period: Cycles) -> Clock {
        Clock { cycle_period: freq.period(), adjust_period, cycles: 0, elapsed: Duration::new(0, 0) }
    }

    pub fn walk(&mut self, cycles: Cycles, elapsed: Duration) {
        self.cycles += cycles;
        self.elapsed += elapsed;

        if self.cycles >= self.adjust_period {
            let expected = self.cycle_period * self.cycles as u32;
            if expected > self.elapsed {
                thread::sleep(expected - self.elapsed);
            } else {
                println!(
                    "Warning: emulated CPU is faster than physical ({}ns late per {} cycles)", 
                    (self.elapsed - expected).subsec_nanos(), 
                    self.cycles,
                );
            }
            self.reset();
        }
    }

    pub fn reset(&mut self) {
        self.cycles = 0;
        self.elapsed = Duration::new(0, 0);
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
}