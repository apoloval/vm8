use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};

pub trait Memory {
    type Addr;

    fn read(&self, addr: Self::Addr, buf: &mut[u8]);
    fn write(&mut self, addr: Self::Addr, buf: &[u8]);

    fn read_i8(&self, addr: Self::Addr) -> i8 {
        let mut buf = [0];
        self.read(addr, &mut buf);
        buf[0] as i8
    }

    fn write_i8(&mut self, addr: Self::Addr, val: i8) {
        let buf = [val as u8];
        self.write(addr, &buf);
    }

    fn read_i16<O: ByteOrder>(&self, addr: Self::Addr) -> i16 {
        let mut buf = [0, 0];
        self.read(addr, &mut buf);
        let mut rbuf: &[u8] = &buf;
        rbuf.read_i16::<O>().unwrap()
    }

    fn write_i16<O: ByteOrder>(&mut self, addr: Self::Addr, val: i16) {
        let mut buf = [0, 0];
        {
            let mut wbuf: &mut [u8] = &mut buf;
            wbuf.write_i16::<O>(val).unwrap();
        }
        self.write(addr, &buf);
    }
}