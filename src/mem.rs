use std::fs;
use std::io::{self, Read};
use std::path::Path;

pub struct ROM {
    bytes: Vec<u8>,
}

impl ROM {
    pub fn load_from_file(path: &Path) -> io::Result<Self> {
        let f = fs::File::open(path)?;
        let mut reader = io::BufReader::new(f);
        let mut buffer = Vec::new();
    
        reader.read_to_end(&mut buffer)?;
        Ok(Self {
            bytes: buffer,
        })
    }

    pub fn read_byte(&self, addr: usize) -> u8 {
        self.bytes[addr]
    }
}
