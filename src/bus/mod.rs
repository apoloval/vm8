use byteorder::{ByteOrder, ReadBytesExt};

pub trait Memory {
    type Addr;

    fn read(&self, addr: Self::Addr, buf: &mut[u8]);

    fn read_i8(&self, addr: Self::Addr) -> i8 {
        let mut buf = [0];
        self.read(addr, &mut buf);
        buf[0] as i8
    }

    fn read_i16<O: ByteOrder>(&self, addr: Self::Addr) -> i16 {
        let mut buf = [0, 0];
        self.read(addr, &mut buf);
        let mut rbuf: &[u8] = &buf;
        rbuf.read_i16::<O>().unwrap()
    }
}