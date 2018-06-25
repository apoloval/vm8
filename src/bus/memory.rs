use std::io;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};

use bus::{Address, Addr16};

pub trait Memory {
    type Addr: Address;

    fn read(&self, addr: Self::Addr, buf: &mut[u8]);
    fn write(&mut self, addr: Self::Addr, buf: &[u8]);
}

pub fn read_from<M: Memory>(mem: &M, from: M::Addr) -> MemoryRead<M> {
    MemoryRead{mem: mem, from: from}
}

pub trait Memory16 : Memory<Addr=Addr16> {}

impl<T: Memory<Addr=Addr16>> Memory16 for T {}

pub struct MemoryRead<'a, M: Memory + 'a> {
    mem: &'a M,
    from: M::Addr,
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
    fn mem_read<M: Memory>(mem: &M, addr: M::Addr) -> Self;
    fn mem_write<M: Memory>(mem: &mut M, addr: M::Addr, val: Self);
}

impl<O: ByteOrder> MemoryItem<O> for i8 {
    fn mem_read<M: Memory>(mem: &M, addr: M::Addr) -> i8 {
        let mut buf = [0];
        mem.read(addr, &mut buf);
        buf[0] as i8
    }

    fn mem_write<M: Memory>(mem: &mut M, addr: M::Addr, val: i8) {
        let buf = [val as u8];
        mem.write(addr, &buf);
    }
}

impl<O: ByteOrder> MemoryItem<O> for i16 {
    fn mem_read<M: Memory>(mem: &M, addr: M::Addr) -> i16 {
        let mut buf = [0, 0];
        mem.read(addr, &mut buf);
        let mut rbuf: &[u8] = &buf;
        rbuf.read_i16::<O>().unwrap()
    }

    fn mem_write<M: Memory>(mem: &mut M, addr: M::Addr, val: i16) {
        let mut buf = [0, 0];
        {
            let mut wbuf: &mut [u8] = &mut buf;
            wbuf.write_i16::<O>(val).unwrap();
        }
        mem.write(addr, &buf);
    }
}