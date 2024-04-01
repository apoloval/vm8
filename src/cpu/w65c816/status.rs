#[cfg(test)] use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Flag {
    C = 0b0000_0001,  // bit 0: carry flag
    Z = 0b0000_0010,  // bit 1: zero flag
    I = 0b0000_0100,  // bit 2: IRQ disable flag
    D = 0b0000_1000,  // bit 3: decimal mode flag
    X = 0b0001_0000,  // bit 4 (native mode): index register select flag
    M = 0b0010_0000,  // bit 5 (native mode): memory select flag
    V = 0b0100_0000,  // bit 6: overflow flag
    N = 0b1000_0000,  // bit 7: negative flag
}

impl Flag {
    pub const B: Flag = Flag::X;  // bit 4 (emulation mode): break flag

    #[inline]
    pub fn mask(self) -> u8 {
        self as u8
    }

    #[inline]
    pub fn set(self, p: &mut u8) {
        *p |= self.mask();
    }

    #[inline]
    pub fn clear(self, p: &mut u8) {
        *p &= !self.mask();
    }
}

#[cfg(test)]
pub struct FlagExpectation(pub Vec<(Flag, bool)>);

#[cfg(test)]
impl FlagExpectation {
    pub fn assert(self, p: u8) {
        for (flag, expected) in self.0 {
            if expected {
                assert_eq!(p & flag.mask(), flag.mask(), "flag {:?} is unexpectedly reset", flag);
            } else {
                assert_eq!(p & flag.mask(), 0, "flag {:?} is unexpectedly set", flag);
            }
        }
    }
}

#[cfg(test)]
impl FromStr for FlagExpectation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut flags = Vec::new();
        if s.is_empty() {
            return Ok(FlagExpectation(flags));
        }

        for prop in s.split(',') {
            let mut parts = prop.split(':');
            match (parts.next(), parts.next()) {
                (Some("C"), Some(val)) =>
                    flags.push((Flag::C, val == "1")),
                (Some("Z"), Some(val)) =>
                    flags.push((Flag::Z, val == "1")),
                (Some("I"), Some(val)) =>
                    flags.push((Flag::I, val == "1")),
                (Some("D"), Some(val)) =>
                    flags.push((Flag::D, val == "1")),
                (Some("X"), Some(val)) =>
                    flags.push((Flag::X, val == "1")),
                (Some("M"), Some(val)) =>
                    flags.push((Flag::M, val == "1")),
                (Some("V"), Some(val)) =>
                    flags.push((Flag::V, val == "1")),
                (Some("N"), Some(val)) =>
                    flags.push((Flag::N, val == "1")),
                (Some("E"), Some(val)) =>
                    flags.push((Flag::N, val == "1")),
                (Some("B"), Some(val)) =>
                    flags.push((Flag::B, val == "1")),
                _ => return Err(format!("Invalid flag expectation: {}", s)),
            }            
        }
        Ok(FlagExpectation(flags))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn flag_mask() {
        assert_eq!(Flag::C.mask(), 0b0000_0001);
        assert_eq!(Flag::Z.mask(), 0b0000_0010);
        assert_eq!(Flag::I.mask(), 0b0000_0100);
        assert_eq!(Flag::D.mask(), 0b0000_1000);
        assert_eq!(Flag::X.mask(), 0b0001_0000);
        assert_eq!(Flag::B.mask(), 0b0001_0000);
        assert_eq!(Flag::M.mask(), 0b0010_0000);
        assert_eq!(Flag::V.mask(), 0b0100_0000);
        assert_eq!(Flag::N.mask(), 0b1000_0000);
    }   
}