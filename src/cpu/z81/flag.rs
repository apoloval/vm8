use std::ops::{Add, BitAnd, Not, Sub};

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

impl Predicate for Flag {
    fn eval(&self, f: u8) -> bool { self.set_mask & f > 0 }
}

impl Not for Flag {
    type Output = Inv<Flag>;
    fn not(self) -> Inv<Flag> { Inv(self) }
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
#[derive(Clone, Copy)]
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

impl BitAnd<Affection> for Affection {
    type Output = Self;
    fn bitand(self, rhs: Affection) -> Self {
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

/// A predicate over some flags.
pub trait Predicate {
    fn eval(&self, f: u8) -> bool;
}

/// A predicate that always evaluates to true.
pub struct Any;

impl Predicate for Any {
    fn eval(&self, _: u8) -> bool { true }
}

/// A predicate that is inverted
pub struct Inv<T: Predicate>(T);

impl<T: Predicate> Predicate for Inv<T> {
    fn eval(&self, f: u8) -> bool { !self.0.eval(f) }
}

/// Return the intrinsic flags for the given value.
/// 
/// Intrinsic flags are those that do not depend on the operation performed, but the result. They are
/// S, Z, F5 and F3. 
pub fn intrinsic(val: u8) -> Affection {
    S.on(signed(val)) & Z.on(val == 0) & intrinsic_undocumented(val)
}

/// Return the intrinsic values of undocumented flags F3 and F5.
pub fn intrinsic_undocumented(val: u8) -> Affection {
    F5.on(val & 0b0010_0000 > 0) & F3.on(val & 0b0000_1000 > 0)
}

#[inline] pub fn carry_nibble(a: u8, c: u8) -> bool { carry(a, c, 0x0F) }
#[inline] pub fn carry_byte(a: u8, c: u8) -> bool { carry(a, c, 0xFF) }
#[inline] pub fn carry_word(a: u16, c: u16) -> bool { carry(a, c, 0xFFFF) }

#[inline]
pub fn carry<T: Into<usize>>(a: T, c: T, mask: usize) -> bool {
    (a.into() & mask) > (c.into() & mask)
}

#[inline] pub fn borrow_nibble(a: u8, c: u8) -> bool { borrow(a, c, 0x0F) }
#[inline] pub fn borrow_byte(a: u8, c: u8) -> bool { borrow(a, c, 0xFF) }

#[inline]
pub fn borrow<T: Into<usize>>(a: T, c: T, mask: usize) -> bool {
    (a.into() & mask) < (c.into() & mask)
}

#[inline] pub fn overflow(a: u8, b: u8, c: u8) -> bool { ((a ^ b ^ 0x80) & (b ^ c) & 0x80) != 0 }
#[inline] pub fn underflow(a: u8, b: u8, c: u8) -> bool {  ((a ^ b) & ((a ^ c) & 0x80)) != 0 }

#[inline] 
pub fn parity(mut c: u8) -> bool {
    let mut ones = 0;
    while c > 0 {
        if c & 0x1 > 0 { ones += 1 }
        c >>= 1;
    }
    ones % 2 == 0
}

#[inline] pub fn signed(c: u8) -> bool { c & 0x80 > 0 }

/// Precomputed flags for unary operators.
pub struct PrecomputedUnary {
    affections: Vec<Affection>,
}

impl PrecomputedUnary {
    /// Return precomputed flags for inc8 operation.
    pub fn for_inc8() -> Self {
        Self::precompute(|i| {
            let a = i;
            let c = i.wrapping_add(1);
            intrinsic(c) & H.on(carry_nibble(a, c)) & V.on(overflow(a, 1, c)) - N
        })
    }

    /// Return precomputed flags for dec8 operation.
    pub fn for_dec8() -> Self {
        Self::precompute(|i| {
            let a = i;
            let c = i.wrapping_sub(1);
            intrinsic(c) & H.on(borrow_nibble(a, c)) & V.on(underflow(a, 1, c)) + N
        })
    }

    /// Return precomputed flags for rla/rlca operations.
    pub fn for_rla() -> Self {
        Self::precompute(|i| {
            let a = i;
            let c = a << 1;
            intrinsic_undocumented(c) & C.on(a & 0x80 > 0) - H - N
        })
    }

    /// Return precomputed flags for rra/rrca operations.
    pub fn for_rra() -> Self {
        Self::precompute(|i| {
            let a = i;
            let c = a >> 1;
            intrinsic_undocumented(c) & C.on(a & 0x01 > 0) - H - N
        })
    }

    fn precompute<F: Fn(u8) -> Affection>(f: F) -> Self {
        let mut affections = Vec::with_capacity(256);
        for i in 0..=255 {
            affections.push(f(i));
        }
        Self { affections }
    }

    /// Return the flags affection for the given operand.
    pub fn for_op(&self, op: u8) -> Affection { self.affections[op as usize] }
}

/// Precomputed flags for binary operators.
pub struct PrecomputedBinary {
    affections: Vec<Affection>,
}

impl PrecomputedBinary {
    /// Return precomputed flags for add8(a, b) operation.
    pub fn for_add8() -> Self {
        Self::precompute(|a, b| {
            let c = a.wrapping_add(b);
            intrinsic(c) & 
                H.on(carry_nibble(a, c)) & 
                V.on(overflow(a, b, c)) & 
                C.on(carry_byte(a, c)) - N
        })
    }

    /// Return precomputed flags for sub8(a, b) operation.
    pub fn for_sub8() -> Self {
        Self::precompute(|a, b| {
            let c = a.wrapping_sub(b);
            intrinsic(c) &
                H.on(borrow_nibble(a, c)) &
                V.on(underflow(a, b, c)) & 
                C.on(borrow_byte(a, c)) + N
        })
    }

    /// Return precomputed flags for and8(a, b) operation.
    pub fn for_and8() -> Self {
        Self::precompute(|a, b| {
            let c = a & b;
            intrinsic(c) &
                P.on(parity(c)) &
                C.on(carry_byte(a, c)) + H - N - C
        })
    }

    /// Return precomputed flags for xor8(a, b) operation.
    pub fn for_xor8() -> Self {
        Self::precompute(|a, b| {
            let c = a ^ b;
            intrinsic(c) &
                P.on(parity(c)) &
                C.on(carry_byte(a, c)) - H - N - C
        })
    }

    /// Return precomputed flags for or8(a, b) operation.
    pub fn for_or8() -> Self {
        Self::precompute(|a, b| {
            let c = a | b;
            intrinsic(c) &
                P.on(parity(c)) &
                C.on(carry_byte(a, c)) - H - N - C
        })
    }

    fn precompute<F: Fn(u8, u8) -> Affection>(f: F) -> Self {
        let mut affections = Vec::with_capacity(256);
        for a in 0..=255 {
            for b in 0..=255 {
                affections.push(f(a, b));
            }
        }
        Self { affections }
    }

    /// Return the flags affection for the given operand.
    pub fn for_ops(&self, a: u8, b: u8) -> Affection { self.affections[a as usize * 256 + b as usize] }
}

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

    #[test]
    fn test_predicate() {
        assert_eq!(S.eval(0b1000_0000), true);
        assert_eq!(Z.eval(0b0100_0000), true);
        assert_eq!(F5.eval(0b0010_0000), true);
        assert_eq!(H.eval(0b0001_0000), true);
        assert_eq!(F3.eval(0b0000_1000), true);
        assert_eq!(PV.eval(0b0000_0100), true);
        assert_eq!(N.eval(0b0000_0010), true);
        assert_eq!(C.eval(0b0000_0001), true);

        assert_eq!(S.eval(0b0000_0000), false);
        assert_eq!(Z.eval(0b0000_0000), false);
        assert_eq!(F5.eval(0b0000_0000), false);
        assert_eq!(H.eval(0b0000_0000), false);
        assert_eq!(F3.eval(0b0000_0000), false);
        assert_eq!(PV.eval(0b0000_0000), false);
        assert_eq!(N.eval(0b0000_0000), false);
        assert_eq!(C.eval(0b0000_0000), false);
    }

    #[test]
    fn test_predicate_not() {
        assert_eq!((!S).eval(0b1000_0000), false);
        assert_eq!((!Z).eval(0b0100_0000), false);
        assert_eq!((!F5).eval(0b0010_0000), false);
        assert_eq!((!H).eval(0b0001_0000), false);
        assert_eq!((!F3).eval(0b0000_1000), false);
        assert_eq!((!PV).eval(0b0000_0100), false);
        assert_eq!((!N).eval(0b0000_0010), false);
        assert_eq!((!C).eval(0b0000_0001), false);

        assert_eq!((!S).eval(0b0000_0000), true);
        assert_eq!((!Z).eval(0b0000_0000), true);
        assert_eq!((!F5).eval(0b0000_0000), true);
        assert_eq!((!H).eval(0b0000_0000), true);
        assert_eq!((!F3).eval(0b0000_0000), true);
        assert_eq!((!PV).eval(0b0000_0000), true);
        assert_eq!((!N).eval(0b0000_0000), true);
        assert_eq!((!C).eval(0b0000_0000), true);
    }

    #[test]
    fn test_parity() {
        assert!(parity(0b0000_0000));
        assert!(parity(0b0000_0011));
        assert!(parity(0b0011_0011));

        assert!(!parity(0b0000_0001));
        assert!(!parity(0b0000_0111));
        assert!(!parity(0b1011_0011));
    }
}