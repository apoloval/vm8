use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};

pub trait Memory {
    type Addr;

    fn read(&self, addr: Self::Addr, buf: &mut[u8]);
    fn write(&mut self, addr: Self::Addr, buf: &[u8]);    
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