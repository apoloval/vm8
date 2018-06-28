use std::time::Duration;

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn frequency_period() {
        let freq = Frequency::from_mhz(3.58);
        assert_eq!(Duration::from_nanos(279), freq.period());
    }
}