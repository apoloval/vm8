use std::cmp;
use std::io;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};

use bus::Address;

pub trait Memory {
    fn read(&self, addr: Address, buf: &mut[u8]);
    fn write(&mut self, addr: Address, buf: &[u8]);
}

pub fn read_from<M: Memory>(mem: &M, from: Address) -> MemoryRead<M> {
    MemoryRead{mem: mem, from: from}
}

pub struct MemoryRead<'a, M: Memory + 'a> {
    mem: &'a M,
    from: Address,
}

impl<'a, M: Memory + 'a> io::Read for MemoryRead<'a, M> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let nbytes = buf.len();
        self.mem.read(self.from, buf);
        self.from = self.from + nbytes;
        Ok(nbytes)
    }
}

pub trait MemoryItem<O: ByteOrder> {
    fn mem_read<M: Memory>(mem: &M, addr: Address) -> Self;
    fn mem_write<M: Memory>(mem: &mut M, addr: Address, val: Self);
}

impl<O: ByteOrder> MemoryItem<O> for u8 {
    fn mem_read<M: Memory>(mem: &M, addr: Address) -> u8 {
        let mut buf = [0];
        mem.read(addr, &mut buf);
        buf[0]
    }

    fn mem_write<M: Memory>(mem: &mut M, addr: Address, val: u8) {
        let buf = [val];
        mem.write(addr, &buf);
    }
}

impl<O: ByteOrder> MemoryItem<O> for u16 {
    fn mem_read<M: Memory>(mem: &M, addr: Address) -> u16 {
        let mut buf = [0, 0];
        mem.read(addr, &mut buf);
        let mut rbuf: &[u8] = &buf;
        rbuf.read_u16::<O>().unwrap()
    }

    fn mem_write<M: Memory>(mem: &mut M, addr: Address, val: u16) {
        let mut buf = [0, 0];
        {
            let mut wbuf: &mut [u8] = &mut buf;
            wbuf.write_u16::<O>(val).unwrap();
        }
        mem.write(addr, &buf);
    }
}

pub struct MemoryBank {
    data: Vec<u8>,
}

impl MemoryBank {
    pub fn with_size(size: usize) -> MemoryBank {
        MemoryBank { data: vec![0; size] }
    }

    pub fn size(&self) -> usize { self.data.len() }
}

impl Memory for MemoryBank {
    fn read(&self, addr: Address, buf: &mut[u8]) {
        let expected = buf.len();
        let actual = {
            let offset = usize::from(addr);
            let limit = cmp::min(self.data.len(), offset + expected);
            let mut input: &[u8] = &self.data[offset..limit];
            let mut output: &mut[u8] = buf;
            io::copy(&mut input, &mut output).unwrap() as usize
        };
        let remaining = expected - actual;
        if remaining > 0 {
            self.read(Address::from(0), &mut buf[actual..]);
        }
    }

    fn write(&mut self, addr: Address, buf: &[u8]) {
        let expected = buf.len();
        let actual = {
            let offset = usize::from(addr);
            let limit = cmp::min(self.data[offset..].len(), buf.len());
            let mut input: &[u8] = &buf[..limit];
            let mut output: &mut[u8] = &mut self.data[offset..];
            io::copy(&mut input, &mut output).unwrap() as usize
        };
        let remaining = expected - actual;
        if remaining > 0 {
            self.write(Address::from(0), &buf[actual..]);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_memory_bank_read() {
        let mut bank = MemoryBank::with_size(64*1024);
        bank.write(Address::from(0x0000), &[0x56, 0x78]);
        bank.write(Address::from(0xfffe), &[0x12, 0x34]);
        let mut buff = [0; 4];
        bank.read(Address::from(0xfffe), &mut buff);
        assert_eq!(buff, [0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn test_memory_bank_write() {
        let mut bank = MemoryBank::with_size(64*1024);
        bank.write(Address::from(0xfffe), &[0x12, 0x34, 0x56, 0x78]);
        let mut buff = [0; 2];
        bank.read(Address::from(0xfffe), &mut buff);
        assert_eq!(buff, [0x12, 0x34]);
        bank.read(Address::from(0x0000), &mut buff);
        assert_eq!(buff, [0x56, 0x78]);
    }
}