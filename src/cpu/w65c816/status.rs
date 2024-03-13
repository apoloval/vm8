#[cfg(test)] use std::str::FromStr;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
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

#[derive(Clone, Copy)]
pub struct Flags {
    set: u8,
    reset: u8,
}

impl Flags {
    pub fn set(flag: Flag) -> Flags {
        Flags { set: flag.mask(), reset: 0 }
    }

    pub fn clear(flag: Flag) -> Flags {
        Flags { set: 0, reset: flag.mask() }
    }

    pub fn and_set(self, flag: Flag) -> Flags {
        Flags { set: flag.mask() | self.set, reset: self.reset }        
    }

    pub fn and_clear(self, flag: Flag) -> Flags {
        Flags { set: self.set, reset: flag.mask() | self.reset }
    }

    pub fn apply(self, p: &mut u8) {
        *p = (*p | self.set) & !self.reset;
    }
}

#[cfg(test)]
pub struct FlagExpectation(pub Vec<(Flag, bool)>);

#[cfg(test)]
impl FlagExpectation {
    pub fn assert(self, p: u8) {
        for (flag, expected) in self.0 {
            if expected {
                assert_eq!(p & flag.mask(), flag.mask());
            } else {
                assert_eq!(p & flag.mask(), 0);
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

    #[test]
    fn flag_affection() {
        let mut p: u8 = 0b0000_0000;
        Flags::set(Flag::C).apply(&mut p);        
        assert_eq!(p, 0b0000_0001);

        p = 0b0000_0001;
        Flags::clear(Flag::C).apply(&mut p);
        assert_eq!(p, 0b0000_0000);

        p = 0b0000_0010;
        Flags::set(Flag::C).and_clear(Flag::Z).apply(&mut p);
        assert_eq!(p, 0b0000_0001);

        p = 0b0000_0001;
        Flags::clear(Flag::C).and_set(Flag::Z).apply(&mut p);
        assert_eq!(p, 0b0000_0010);
    }
}