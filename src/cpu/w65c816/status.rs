pub enum Flag {
    C = 0b0000_0001,  // bit 0: carry flag
    Z = 0b0000_0010,  // bit 1: zero flag
    I = 0b0000_0100,  // bit 2: IRQ disable flag
    D = 0b0000_1000,  // bit 3: decimal mode flag
    X = 0b0001_0000,  // bit 4 (native mode): index register select flag
    M = 0b0010_0000,  // bit 5 (native mode): memory select flag
    O = 0b0100_0000,  // bit 6: overflow flag
    N = 0b1000_0000,  // bit 7: negative flag
}

impl Flag {
    pub const B: Flag = Flag::X;  // bit 4 (emulation mode): break flag

    pub fn mask(self) -> u8 {
        self as u8
    }

    pub fn set(self, p: &mut u8) {
        *p |= self.mask();
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
        assert_eq!(Flag::O.mask(), 0b0100_0000);
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