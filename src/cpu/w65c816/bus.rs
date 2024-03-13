#[cfg(test)] use std::str::FromStr;

/// An address used by the W65C816 CPU buses. In essence, it is a 24-bit value, with the upper 8
/// bits representing the bank and the lower 16 bits representing the offset within the bank.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Addr {
    raw: usize,
}

impl Addr {
    /// Creates a new Addr value from a bank and an offset.
    pub fn from(bank: u8, offset: u16) -> Self {
        Self { raw: (bank as usize) << 16 | offset as usize }
    }

    /// Add the given magnitude to the address, wrapping around the address space if necessary.
    #[inline]
    pub fn wrapping_add<T: Into<usize>>(self, other: T, wrap: AddrWrap) -> Self {
        match wrap {
            AddrWrap::Byte => Self { 
                raw: (self.raw & 0xFFFF00) | (self.raw.wrapping_add(other.into()) & 0xFF),
            },
            AddrWrap::Word => Self { 
                raw: (self.raw & 0xFF0000) | (self.raw.wrapping_add(other.into()) & 0xFFFF),
            },
            AddrWrap::Long => Self {
                raw: self.raw.wrapping_add(other.into()) & 0xFFFFFF,            
            }
        }
    }

    /// Increment the address by one, wrapping around the address space if necessary.
    #[inline]
    pub fn inc(self, wrap: AddrWrap) -> Self {
        self.wrapping_add(1usize, wrap)
    }

    /// Returns true if the two addresses are in the same page.
    pub fn same_page(&self, other: Self) -> bool {
        self.raw & 0xFFFF00 == other.raw & 0xFFFF00
    }
}

impl From<Addr> for usize {
    fn from(addr: Addr) -> usize {
        addr.raw
    }
}

/// Indicates how an Addr value must wrap while adding a value. Allowed vlaues are Byte, Word and
/// Long for 8-bit byte, 16-bit word or 24-bit long wrappign respectively.
#[derive(Debug, Clone, Copy)]
pub enum AddrWrap { Byte, Word, Long }    

pub trait Bus {
    fn read_byte(&self, addr: Addr) -> u8;
    fn write_byte(&mut self, addr: Addr, val: u8);

    #[inline]
    fn read_word(&self, addr: Addr, wrap: AddrWrap) -> u16 {
        let lo = self.read_byte(addr) as u16;
        let hi = self.read_byte(addr.inc(wrap)) as u16;
        (hi << 8) | lo
    }

    #[inline]
    fn write_word(&mut self, addr: Addr, wrap: AddrWrap, val: u16) {
        let lo = val as u8;
        let hi = (val >> 8) as u8;
        self.write_byte(addr, lo);
        self.write_byte(addr.inc(wrap), hi);
    }
}

impl Bus for () {
    fn read_byte(&self, _: Addr) -> u8 { 0xFF }
    fn write_byte(&mut self, _: Addr, _: u8) {}
}

#[cfg(test)]
pub struct Fake {
    banks: Vec<u8>
}

#[cfg(test)]
impl Fake {
    pub fn new() -> Self {
        Self {
            banks: vec![0; 256*64*1024],
        }
    }
}

#[cfg(test)]
impl Bus for Fake {
    fn read_byte(&self, addr: Addr) -> u8 { 
        self.banks[usize::from(addr)]
    }

    fn write_byte(&mut self, addr: Addr, val: u8) { 
        self.banks[usize::from(addr)] = val 
    }
}

#[cfg(test)]
impl FromStr for Fake {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bus = Self::new();

        if s == "" {
            return Ok(bus);
        }

        for prop in s.split(',') {
            let mut parts = prop.split(':');
            let left = parts.next().ok_or(format!("invalid syntax in prop: {}", prop))?;
            let mut right = parts.next().ok_or(format!("invalid value in prop: {}", prop))?;

            let bank = u8::from_str_radix(&left[0..2], 16)
                .map_err(|e| format!("invalid bank value: {}", e))?;
            let offset = u16::from_str_radix(&left[2..], 16)
                .map_err(|e| format!("invalid addr value: {}", e))?;

            let mut addr = Addr::from(bank, offset);

            while right.len() > 0 {
                let byte = u8::from_str_radix(&right[0..2], 16)
                    .map_err(|e| format!("invalid byte value: {}", e))?;
                bus.write_byte(addr, byte);
                addr = addr.inc(AddrWrap::Long);
                right = &right[2..];
            }
        }
        Ok(bus)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn addr_wrapping_add_byte() {
        let addr = Addr::from(0xDD, 0xEEFF);
        let addr = addr.wrapping_add(0x01u8, AddrWrap::Byte);
        assert_eq!(addr.raw, 0xDDEE00);
    }

    #[test]
    fn addr_wrapping_add_word() {
        let addr = Addr::from(0xDD, 0xFFFF);
        let addr = addr.wrapping_add(0x01u16, AddrWrap::Word);
        assert_eq!(addr.raw, 0xDD0000);
    }

    #[test]
    fn addr_wrapping_add_long() {
        let addr = Addr::from(0xFF, 0xFFFF);
        let addr = addr.wrapping_add(0x01usize, AddrWrap::Long);
        assert_eq!(addr.raw, 0x000000);
    }
}