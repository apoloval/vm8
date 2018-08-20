use std::io;

use byteorder::ByteOrder;

use bus::Address;

pub trait Memory {
    fn read_byte(&self, addr: Address) -> u8;
    fn write_byte(&mut self, addr: Address, val: u8);

    fn read_word<O: ByteOrder>(&self, addr: Address) -> u16 {
        let data = [self.read_byte(addr), self.read_byte(addr + 1)];
        O::read_u16(&data)
    }

    fn write_word<O: ByteOrder>(&mut self, addr: Address, val: u16) {
        let mut data = [0; 2];
        O::write_u16(&mut data, val);
        self.write_byte(addr, data[0]);
        self.write_byte(addr + 1, data[1]);
    }
}

pub struct MemoryBank {
    data: Vec<u8>,
    readonly: bool,
    addr_mask: usize,
}

impl MemoryBank {
    pub fn with_size(size: usize) -> MemoryBank {
        MemoryBank { data: vec![0; size], readonly: false, addr_mask: Self::addr_mask_from_size(size) }
    }

    pub fn size(&self) -> usize { self.data.len() }
    pub fn set_readonly(&mut self, val: bool) { self.readonly = val; }

    pub fn set_data(&mut self, data: &[u8]) -> io::Result<u64> {
        let mut input = data;
        let output = &mut self.data;
        io::copy(&mut input, output)
    }

    fn addr_mask_from_size(mut size: usize) -> usize {
        let mut mask = 0;
        while (size - 1) > 0 {
            size = size >> 1;
            mask = (mask << 1) | 1;
        }
        mask
    }
}

impl Memory for MemoryBank {
    fn read_byte(&self, addr: Address) -> u8 {
        let offset = usize::from(addr) & self.addr_mask;
        self.data[offset]
    }

    fn write_byte(&mut self, addr: Address, val: u8) {
        let offset = usize::from(addr) & self.addr_mask;
        self.data[offset] = val;
    }    
}

pub trait MemoryController {
    fn bank(&self, addr: Address) -> Option<&MemoryBank>;
    fn bank_mut(&mut self, addr: Address) -> Option<&mut MemoryBank>;
}

impl<M: MemoryController> Memory for M {
    fn read_byte(&self, addr: Address) -> u8 {
        match self.bank(addr) {
            Some(bank) => bank.read_byte(addr),
            None => 0,
        }
    }

    fn write_byte(&mut self, addr: Address, val: u8) {
        if let Some(bank) = self.bank_mut(addr) {
            bank.write_byte(addr, val);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use byteorder::BigEndian;

    #[test]
    fn test_memory_bank() {
        let mut bank = MemoryBank::with_size(64*1024);
        bank.write_word::<BigEndian>(Address::from(0x0000), 0x5678);
        bank.write_word::<BigEndian>(Address::from(0xfffe), 0x1234);
        assert_eq!(0x1234, bank.read_word::<BigEndian>(Address::from(0xfffe)));
        assert_eq!(0x5678, bank.read_word::<BigEndian>(Address::from(0x0000)));
        assert_eq!(0x3456, bank.read_word::<BigEndian>(Address::from(0xffff)));
    }
}

#[cfg(all(feature = "nightly", test))]
mod bench {
    use super::*;

    use test;
    use test::Bencher;

    use byteorder::NativeEndian;

    #[bench]
    fn bench_memory_bank_read_64kb_by_byte(b: &mut Bencher) {
        let bank = MemoryBank::with_size(64*1024);
        b.iter(|| {
            for i in 0..0xffff {
                test::black_box(bank.read_byte(Address::from(i)));
            }
        });
    }

    #[bench]
    fn bench_memory_bank_read_64kb_by_word(b: &mut Bencher) {
        let bank = MemoryBank::with_size(64*1024);
        b.iter(|| {
            for i in 0..0xffff {
                test::black_box(bank.read_word::<NativeEndian>(Address::from(i)));
            }
        });
    }

    #[bench]
    fn bench_memory_controller_read_64kb_by_byte(b: &mut Bencher) {
        let bank = MultiBankMemory::new();
        b.iter(|| {
            for i in 0..0xffff {
                test::black_box(bank.read_byte(Address::from(i)));
            }
        });
    }

    #[bench]
    fn bench_memory_controller_read_64kb_by_word(b: &mut Bencher) {
        let bank = MultiBankMemory::new();
        b.iter(|| {
            for i in 0..0xffff {
                test::black_box(bank.read_word::<NativeEndian>(Address::from(i)));
            }
        });
    }

    struct MultiBankMemory {
        rom: MemoryBank,
        ram: MemoryBank,
    }

    impl MultiBankMemory {
        fn new() -> Self {
            let rom = MemoryBank::with_size(16 * 1024);
            let ram = MemoryBank::with_size(64 * 1024);
            Self { rom, ram }
        }
    }

    impl MemoryController for MultiBankMemory {
        fn bank(&self, addr: Address) -> Option<&MemoryBank> {
            match usize::from(addr) {
                0x0000 ... 0x3fff => Some(&self.rom),
                0x4000 ... 0xffff => Some(&self.ram),
                _ => None,
            }
        }

        fn bank_mut(&mut self, addr: Address) -> Option<&mut MemoryBank> {
            match usize::from(addr) {
                0x4000 ... 0xffff => Some(&mut self.ram),
                _ => None,
            }
        }
    }
}