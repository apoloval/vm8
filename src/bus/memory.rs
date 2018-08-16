use std::io;
use std::marker::PhantomData;

use byteorder::{ByteOrder, NativeEndian, ReadBytesExt};

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

pub fn read_from<O: ByteOrder, M: Memory>(mem: &M, from: Address) -> ReadFrom<M, O> {
    ReadFrom{mem: mem, from: from, order: PhantomData }
}

pub trait BurstRead {
    fn read_byte(&mut self) -> u8;
    fn read_word(&mut self) -> u16;
}

impl<R> BurstRead for R where R: io::Read {
    fn read_byte(&mut self) -> u8 {
        self.read_u8().unwrap()
    }

    fn read_word(&mut self) -> u16 {
        self.read_u16::<NativeEndian>().unwrap()
    }
}

pub struct ReadFrom<'a, M: Memory + 'a, O: ByteOrder> {
    mem: &'a M,
    from: Address,
    order: PhantomData<O>,
}

impl<'a, M: Memory + 'a, O: ByteOrder> BurstRead for ReadFrom<'a, M, O> {
    fn read_byte(&mut self) -> u8 {
        let byte = self.mem.read_byte(self.from);
        self.from = self.from + 1;
        byte
    }

    fn read_word(&mut self) -> u16 {
        let word = self.mem.read_word::<O>(self.from);
        self.from = self.from + 2;
        word
    }
}

pub trait MemoryItem<O: ByteOrder> {
    fn mem_read<M: Memory>(mem: &M, addr: Address) -> Self;
    fn mem_write<M: Memory>(mem: &mut M, addr: Address, val: Self);
}

impl<O: ByteOrder> MemoryItem<O> for u8 {
    fn mem_read<M: Memory>(mem: &M, addr: Address) -> u8 {
        mem.read_byte(addr)
    }

    fn mem_write<M: Memory>(mem: &mut M, addr: Address, val: u8) {
        mem.write_byte(addr, val);
    }
}

impl<O: ByteOrder> MemoryItem<O> for u16 {
    fn mem_read<M: Memory>(mem: &M, addr: Address) -> u16 {
        mem.read_word::<O>(addr)
    }

    fn mem_write<M: Memory>(mem: &mut M, addr: Address, val: u16) {
        mem.write_word::<O>(addr, val);
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