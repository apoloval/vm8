use std::fmt;
use std::fs;
use std::io::{self, Read};

pub enum Command {
    Help,
    Exit,
    Regs,
    Step,
    MemRead { addr: Option<u16> },
    MemWrite { addr: u16, data: Vec<u8> },
}

#[derive(Debug)]
pub enum ParseError {
    UnknownCommand,
    InvalidParameter,
    Io(io::Error),
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> Self { ParseError::Io(err) }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> { 
        match self {
            ParseError::UnknownCommand => write!(f, "unknown command"),
            ParseError::InvalidParameter => write!(f, "invalid parameter"),
            ParseError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl Command {
    pub fn parse(line: &str) -> Result<Command, ParseError> {
        let mut params = line.split_whitespace();
        match params.next() {
            Some("help") | Some("?") => Ok(Command::Help),
            Some("exit") | Some("x") => Ok(Command::Exit),
            Some("regs") | Some("r") => Ok(Command::Regs),
            Some("step") | Some("s") => Ok(Command::Step),
            Some("memread") => Self::parse_memread(params),
            Some("memwrite") => Self::parse_memwrite(params),
            _ => Err(ParseError::UnknownCommand),
        }
    }

    pub fn print_help() {
        println!("Commands:");
        println!("  help | ?                Print this help");
        println!("  exit | x                Exit and return to shell");
        println!("  regs | r                Print status of CPU registers");
        println!("  step | s                Execute one CPU step");
        println!("  memread [<addr>]        Print memory content starting at <addr>[default:PC]");
        println!("  memwrite <addr> <data>  Write data into memory at given address");
        println!("");
        println!("Data formats:");
        println!("  data:[<byte>]+           Literal data with bytes in hexadecimal");
        println!("  file:[<byte>]+           Literal data with bytes in hexadecimal");
    }

    fn parse_memread<'a, I: Iterator<Item=&'a str>>(mut params: I) -> Result<Command, ParseError> {
        match params.next() {
            Some(addr) => Self::parse_addr(addr).map(|a| Command::MemRead { addr: Some(a) }),
            None => Ok(Command::MemRead { addr: None }),
        }
    }

    fn parse_memwrite<'a, I: Iterator<Item=&'a str>>(mut params: I) -> Result<Command, ParseError> {
        let addr = params.next().ok_or(ParseError::InvalidParameter).and_then(Self::parse_addr)?;
        let data = params.next().ok_or(ParseError::InvalidParameter).and_then(Self::parse_data)?;
        Ok(Command::MemWrite { addr, data })
    }

    fn parse_addr(s: &str) -> Result<u16, ParseError> {
        u16::from_str_radix(s, 16).or(Err(ParseError::InvalidParameter))
    }

    fn parse_data(s: &str) -> Result<Vec<u8>, ParseError> {
        if let Some(data) = s.strip_prefix("data:") {
            return Self::parse_data_literal(data);
        }
        if let Some(file) = s.strip_prefix("file:") {
            return Self::parse_data_file(file);
        }
        Err(ParseError::InvalidParameter)
    }

    fn parse_data_literal(s: &str) -> Result<Vec<u8>, ParseError> {
        let mut bytes = Vec::with_capacity(s.len() / 2);
        for i in (0..s.len()).step_by(2) {
            if let Some(byte) = u8::from_str_radix(&s[i..i+2], 16).ok() {
                bytes.push(byte);
            } else {
                return Err(ParseError::InvalidParameter)
            }
        }
        Ok(bytes)
    }

    fn parse_data_file(s: &str) -> Result<Vec<u8>, ParseError> {
        let f = fs::File::open(s)?;
        let mut reader = io::BufReader::new(f);
        let mut buffer = Vec::new();
    
        reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}
