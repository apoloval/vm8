use std::ops::{Add, Sub};

/// A flag used in the Z80 processor. 
#[derive(Copy, Clone)]
pub struct Flag { pub set_mask: u8, pub reset_mask: u8 }

impl Flag {
    /// Return the affection for this flag depending on the given condition.
    pub fn on(self, cond: bool) -> Affection {
        let aff = Affection::default();
        if cond { aff + self }
        else { aff - self }
    }
}

/// Sign flag, typically set when result has its 7th bit (sign) is set. 
pub const S: Flag   = Flag { set_mask: 0b1000_0000, reset_mask: 0b0111_1111};

/// Zero flag, typically set when result is zero. 
pub const Z: Flag   = Flag { set_mask: 0b0100_0000, reset_mask: 0b1011_1111};

/// F5 flag, typically copied from the 5-th bit of the result.
pub const F5: Flag  = Flag { set_mask: 0b0010_0000, reset_mask: 0b1101_1111};

/// Half-carry flag, typically set on carry/borrow between low and high nibbles.
pub const H: Flag   = Flag { set_mask: 0b0001_0000, reset_mask: 0b1110_1111};

/// F3 flag, typically copied from the 3-th bit of the result.
pub const F3: Flag  = Flag { set_mask: 0b0000_1000, reset_mask: 0b1111_0111};

/// Parity/oVerflow flag, typically set when parity or overflow is detected. 
pub const PV: Flag  = Flag { set_mask: 0b0000_0100, reset_mask: 0b1111_1011};

/// Subtract flag, typically set when last operation was a subtraction. 
pub const N: Flag   = Flag { set_mask: 0b0000_0010, reset_mask: 0b1111_1101};

/// Carry flag, typically set when result doesn't fit in the destination.
pub const C: Flag   = Flag { set_mask: 0b0000_0001, reset_mask: 0b1111_1110};

/// An alias of PV to remark it is used for parity.
pub const P: Flag = PV;

/// An alias of PV to remark it is used for overflow.
pub const V: Flag = PV;

/// A carrier of how flags will be affected by some operation.
/// 
/// This is used to describe how flags will be affected after some ALU operation. 
/// A flag can be not-affected, set or reset. This type maintains internal binary 
/// masks to keep track of the three possible values for each flag.
/// 
/// Affections are typically combined using add and sub operators, with either
/// other flags or other affections.
pub struct Affection { set: u8, reset: u8 }

impl Default for Affection {
    /// Default affection is all flags are not-affected. 
    fn default() -> Self { Self { set: 0x00, reset: 0xFF }}
}

impl Add<Flag> for Affection {
    type Output = Self;
    fn add(self, rhs: Flag) -> Self {
        Self {
            set: self.set | rhs.set_mask,
            reset: self.reset | rhs.set_mask,
        }
    }
}

impl Add<Affection> for Affection {
    type Output = Self;
    fn add(self, rhs: Affection) -> Self {
        Self {
            set: rhs.set | (self.set & rhs.reset),
            reset: rhs.reset & (self.reset | rhs.set),
        }
    }
}

impl Sub<Flag> for Affection {
    type Output = Self;
    fn sub(self, rhs: Flag) -> Self {
        Self {
            set: self.set & rhs.reset_mask,
            reset: self.reset & rhs.reset_mask,
        }
    }
}

impl Affection {
    /// Apply this flag affection to the given flags value, and return the resulting flags.
    pub fn apply(&self, val: u8) -> u8 {
        (val | self.set) & self.reset
    }
}

/// Return the intrinsic flags for the given value.
/// 
/// Intrinsic flags are those that do not depend on the operation performed, but the result. They are
/// S, Z, F5 and F3. 
pub fn intrinsic(val: u8) -> Affection {
    S.on(val & 0x80 > 0) + Z.on(val == 0) + F5.on(val & 0b0010_0000 > 0) + F3.on(val & 0b0000_1000 > 0)
}

#[inline] pub fn carry_nibble(a: u8, c: u8) -> bool { carry(a, c, 0x0F) }
#[inline] pub fn carry_byte(a: u8, c: u8) -> bool { carry(a, c, 0xFF) }
#[inline] pub fn carry_word(a: u16, c: u16) -> bool { carry(a, c, 0xFFFF) }

#[inline]
fn carry<T: Into<usize>>(a: T, c: T, mask: usize) -> bool {
    (a.into() & mask) > (c.into() & mask)
}

#[inline] pub fn borrow_nibble(a: u8, c: u8) -> bool { borrow(a, c, 0x0F) }
#[inline] pub fn borrow_byte(a: u8, c: u8) -> bool { borrow(a, c, 0xFF) }
#[inline] pub fn borrow_word(a: u16, c: u16) -> bool { borrow(a, c, 0xFFFF) }

#[inline]
fn borrow<T: Into<usize>>(a: T, c: T, mask: usize) -> bool {
    (a.into() & mask) < (c.into() & mask)
}

#[inline] pub fn overflow(a: u8, b: u8, c: u8) -> bool { ((a ^ b ^ 0x80) & (b ^ c) & 0x80) != 0 }
#[inline] pub fn underflow(a: u8, b: u8, c: u8) -> bool {  ((a ^ b) & ((a ^ c) & 0x80)) != 0 }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_flag_add() {
        let flags = Affection::default() + S + Z;
        assert_eq!(flags.apply(0b0000_0000), 0b1100_0000);
        assert_eq!(flags.apply(0b1000_0000), 0b1100_0000);
        assert_eq!(flags.apply(0b0100_1000), 0b1100_1000);
    }

    #[test]
    fn test_flag_sub() {
        let flags = Affection::default() - S - Z;
        assert_eq!(flags.apply(0b0000_0000), 0b0000_0000);
        assert_eq!(flags.apply(0b1000_0000), 0b0000_0000);
        assert_eq!(flags.apply(0b0100_1000), 0b0000_1000);
    }

    #[test]
    fn test_flag_add_sub() {
        let flags = Affection::default() + S - S;
        assert_eq!(flags.apply(0b0000_0000), 0b0000_0000);
        assert_eq!(flags.apply(0b1000_0000), 0b0000_0000);
        assert_eq!(flags.apply(0b0100_0000), 0b0100_0000);
        assert_eq!(flags.apply(0b1100_0000), 0b0100_0000);
    }

    #[test]
    fn test_flag_sub_add() {
        let flags = Affection::default() - S + S;
        assert_eq!(flags.apply(0b0000_0000), 0b1000_0000);
        assert_eq!(flags.apply(0b1000_0000), 0b1000_0000);
        assert_eq!(flags.apply(0b0100_0000), 0b1100_0000);
        assert_eq!(flags.apply(0b1100_0000), 0b1100_0000);
    }
}