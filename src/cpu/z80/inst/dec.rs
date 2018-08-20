use byteorder::LittleEndian;

use bus::{Address, Memory};
use cpu::z80::inst::Inst;

pub fn decode<M: Memory>(mem: &M, addr: Address) -> Inst {
    let opcode = mem.read_byte(addr);
    match opcode {
        0x00 => { inst!(NOP) },
        0x01 => { inst!(LD BC, mem.read_word::<LittleEndian>(addr + 1)) },
        0x02 => { inst!(LD (BC), A) },
        0x03 => { inst!(INC BC) },
        0x04 => { inst!(INC B) },
        0x05 => { inst!(DEC B) },
        0x06 => { inst!(LD B, mem.read_byte(addr + 1)) },
        0x07 => { inst!(RLCA) },
        0x08 => { inst!(EX AF, AF_) },
        0x09 => { inst!(ADD HL, BC) },
        0x0a => { inst!(LD A, (BC)) },
        0x0b => { inst!(DEC BC) },
        0x0c => { inst!(INC C) },
        0x0d => { inst!(DEC C) },
        0x0e => { inst!(LD C, mem.read_byte(addr + 1)) },
        0x0f => { inst!(RRCA) },
        0xc3 => { inst!(JP mem.read_word::<LittleEndian>(addr + 1)) },               
        _ => { unimplemented!("decoded not implemented for the given opcode"); },        
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use bus::MemoryBank;

    #[test]
    fn should_encode() {
        let tests = [
            DecodeTest {
                what: "NOP",
                input: vec![0x00],
                expected: inst!(NOP),
            },
            DecodeTest {
                what: "LD BC,1234h",
                input: vec![0x01, 0x34, 0x12],
                expected: inst!(LD BC, 0x1234), 
            },
            DecodeTest {
                what: "LD (BC),A",
                input: vec![0x02],
                expected: inst!(LD (BC), A), 
            },
            DecodeTest {
                what: "INC BC",
                input: vec![0x03],
                expected: inst!(INC BC), 
            },
            DecodeTest {
                what: "INC B",
                input: vec![0x04],
                expected: inst!(INC B), 
            },
            DecodeTest {
                what: "DEC B",
                input: vec![0x05],
                expected: inst!(DEC B), 
            },
            DecodeTest {
                what: "LD B,12h",
                input: vec![0x06, 0x12],
                expected: inst!(LD B, 0x12), 
            },
            DecodeTest {
                what: "RLCA",
                input: vec![0x07],
                expected: inst!(RLCA), 
            },
            DecodeTest {
                what: "EX AF,AF'",
                input: vec![0x08],
                expected: inst!(EX AF, AF_), 
            },
            DecodeTest {
                what: "ADD HL,BC'",
                input: vec![0x09],
                expected: inst!(ADD HL, BC), 
            },
            DecodeTest {
                what: "LD A,(BC)'",
                input: vec![0x0a],
                expected: inst!(LD A, (BC)), 
            },
            DecodeTest {
                what: "DEC BC'",
                input: vec![0x0b],
                expected: inst!(DEC BC), 
            },
            DecodeTest {
                what: "INC C'",
                input: vec![0x0c],
                expected: inst!(INC C), 
            },
            DecodeTest {
                what: "DEC C'",
                input: vec![0x0d],
                expected: inst!(DEC C), 
            },
            DecodeTest {
                what: "LD C,12h'",
                input: vec![0x0e, 0x12],
                expected: inst!(LD C, 0x12), 
            },
            DecodeTest {
                what: "RRCA",
                input: vec![0x0f],
                expected: inst!(RRCA), 
            },
            DecodeTest {
                what: "JP 1234h",
                input: vec![0xc3, 0x34, 0x12],
                expected: inst!(JP 0x1234), 
            },
        ];
        for test in &tests {
            test.run();
        }
    }

    struct DecodeTest {
        what: &'static str,
        input: Vec<u8>,
        expected: Inst,
    }

    impl DecodeTest {
        fn run(&self) {
            let mut mem = MemoryBank::with_size(64*1024);
            mem.set_data(&self.input).unwrap();
            let given = decode(&mem, Address::from(0));
            assert_eq!(self.expected, given, "decoding instruction:Dest {}", self.what);
        }
    }
}

#[cfg(all(feature = "nightly", test))] 
mod bench { 
    use super::*; 
 
    use test; 
    use test::Bencher; 

    use bus::{Address, MemoryBank};
 
    #[bench] 
    fn bench_decode_100_1byte_instructions(b: &mut Bencher) { 
        let input = vec![0x00]; 
        let mut mem = MemoryBank::with_size(64*1024);
        mem.set_data(&input).unwrap();
        b.iter(||{ 
            for _ in 1..100 { 
                test::black_box(decode(&mem, Address::from(0))); 
            } 
        }) 
    }     
 
    #[bench] 
    fn bench_decode_100_2byte_instructions(b: &mut Bencher) { 
        let input = vec![0x0e, 0x12]; 
        let mut mem = MemoryBank::with_size(64*1024);
        mem.set_data(&input).unwrap();
        b.iter(||{ 
            for _ in 1..100 { 
                test::black_box(decode(&mem, Address::from(0))); 
            } 
        }) 
    }     
} 
