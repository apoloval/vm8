use std::fmt;
use std::fs;
use std::io::{self, Read};

pub enum Command {
    Help,
    Exit,
    Status,
    Step,
    Resume,
    Reset,
    BreakSet { addr: u16 },
    BreakShow,
    BreakDelete { addr: Option<u16> },
    MemRead { addr: Option<u32> },
    MemWrite { addr: u32, data: Vec<u8> },
}

#[derive(Debug)]
pub enum ParseError {
    NoInput,
    NotEnoughParameters,
    UnknownCommand(String),
    InvalidParameter(String),
    Io(io::Error),
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> Self { ParseError::Io(err) }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> { 
        match self {
            ParseError::NoInput => write!(f, "no input given"),
            ParseError::NotEnoughParameters => write!(f, "not enough parameters"),
            ParseError::UnknownCommand(cmd) => write!(f, "unknown command '{}'", cmd),
            ParseError::InvalidParameter(param) => write!(f, "invalid parameter '{}'", param),
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
            Some("status") | Some("st") => Ok(Command::Status),
            Some("step") | Some("s") => Ok(Command::Step),
            Some("resume") | Some("r") => Ok(Command::Resume),
            Some("break") | Some("b") => Self::parse_break(params),
            Some("delete") => Self::parse_delete(params),
            Some("show") => Self::parse_show(params),
            Some("reset")  => Ok(Command::Reset),
            Some("memread") => Self::parse_memread(params),
            Some("memwrite") => Self::parse_memwrite(params),
            Some(other) => Err(ParseError::UnknownCommand(String::from(other))),
            None => Err(ParseError::NoInput),
        }
    }

    pub fn print_help() {
        println!("Commands:");
        println!("  help | ?                        Print this help");
        println!("  exit | x                        Exit and return to shell");
        println!("  status | s                      Print status of the system");
        println!("  step | s                        Execute one CPU step");
        println!("  resume | r                      Resume the execution");
        println!("  break <addr> | b                Set breakpoint at <addr>");
        println!("  delete [<addr>]                 Delete breakpoints, or that at <addr>");
        println!("  show break                      Show defined breakpoints");
        println!("  reset                           Reset the system");
        println!("  memread [<addr>]                Print memory at <addr> [default:PC]");
        println!("  memwrite <addr> <data>          Write data into memory at given address");
        println!("");
        println!("Data formats:");
        println!("  data:[<byte>]+           Literal data with bytes in hexadecimal");
        println!("  file:[<byte>]+           Literal data with bytes in hexadecimal");
    }

    fn parse_show<'a, I: Iterator<Item=&'a str>>(mut params: I) -> Result<Command, ParseError> {
        let what = params.next().ok_or(ParseError::NotEnoughParameters)?;
        match what {
            "break" => Ok(Command::BreakShow),
            other => Err(ParseError::InvalidParameter(String::from(other))),
        }
        
    }

    fn parse_break<'a, I: Iterator<Item=&'a str>>(mut params: I) -> Result<Command, ParseError> {
        let addr = params.next().ok_or(ParseError::NotEnoughParameters).and_then(Self::parse_addr)?;
        Ok(Command::BreakSet { addr: addr as u16 })
    }

    fn parse_delete<'a, I: Iterator<Item=&'a str>>(mut params: I) -> Result<Command, ParseError> {
        match params.next() {
            Some(addr) => Self::parse_addr(addr).map(|a| Command::BreakDelete { addr: Some(a as u16) }),
            None => Ok(Command::BreakDelete { addr: None }),
        }
    }

    fn parse_memread<'a, I: Iterator<Item=&'a str>>(mut params: I) -> Result<Command, ParseError> {
        match params.next() {
            Some(addr) => Self::parse_addr(addr).map(|a| Command::MemRead { addr: Some(a) }),
            None => Ok(Command::MemRead { addr: None }),
        }
    }

    fn parse_memwrite<'a, I: Iterator<Item=&'a str>>(mut params: I) -> Result<Command, ParseError> {
        let addr = params.next().ok_or(ParseError::NotEnoughParameters).and_then(Self::parse_addr)?;
        let data = params.next().ok_or(ParseError::NotEnoughParameters).and_then(Self::parse_data)?;
        Ok(Command::MemWrite { addr, data })
    }

    fn parse_addr(s: &str) -> Result<u32, ParseError> {
        u32::from_str_radix(s, 16).or(Err(ParseError::InvalidParameter(String::from(s))))
    }

    fn parse_data(s: &str) -> Result<Vec<u8>, ParseError> {
        if let Some(data) = s.strip_prefix("data:") {
            return Self::parse_data_literal(data);
        }
        if let Some(file) = s.strip_prefix("file:") {
            return Self::parse_data_file(file);
        }
        Err(ParseError::InvalidParameter(String::from(s)))
    }

    fn parse_data_literal(s: &str) -> Result<Vec<u8>, ParseError> {
        let mut bytes = Vec::with_capacity(s.len() / 2);
        for i in (0..s.len()).step_by(2) {
            if let Some(byte) = u8::from_str_radix(&s[i..i+2], 16).ok() {
                bytes.push(byte);
            } else {
                return Err(ParseError::InvalidParameter(String::from(s)))
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
