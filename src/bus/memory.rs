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