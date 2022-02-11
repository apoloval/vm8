use std::fmt;
use std::io;

use crate::sys::nexus::Addr;

pub enum Command {
    Help,
    Exit,
    StatusShow,
    Step,
    Resume,
    Reset,
    BreakSet { addr: Addr },
    BreakShow,
    BreakDelete { addr: Option<Addr> },
    MemShow { addr: Option<Addr> },
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
            Some("step") | Some("s") => Ok(Command::Step),
            Some("resume") | Some("r") => Ok(Command::Resume),
            Some("break") | Some("b") => Self::parse_break(params),
            Some("delete") => Self::parse_delete(params),
            Some("show") => Self::parse_show(params),
            Some("st") => Ok(Command::StatusShow),
            Some("m") => Self::parse_show_mem(params),
            Some("reset")  => Ok(Command::Reset),
            Some(other) => Err(ParseError::UnknownCommand(String::from(other))),
            None => Err(ParseError::NoInput),
        }
    }

    pub fn print_help() {
        println!("Breakpoint commands:");
        println!("  break <addr> | b                Set breakpoint at <addr>");
        println!("  delete [<addr>]                 Delete breakpoints, or that at <addr>");
        println!("  show break                      Show defined breakpoints");
        println!("Runtime behavior commands:");
        println!("  resume | r                      Resume the execution");
        println!("  reset                           Reset the system");
        println!("  step | s                        Execute one CPU step");
        println!("  show status | st                Show status of the system");
        println!("  show mem [<addr>] | m           Show memory at <addr> [default:PC]");
        println!("Program control commands:");
        println!("  help | ?                        Print this help");
        println!("  exit | x                        Exit and return to shell");
        println!("");
        println!("Data formats:");
        println!("  <addr>=[0-9A-F]{{1,4}}          A logical address");
        println!("  <addr>=:[0-9A-F]{{1,5}}         A physical address");
    }

    fn parse_show<'a, I: Iterator<Item=&'a str>>(mut params: I) -> Result<Command, ParseError> {
        let what = params.next().ok_or(ParseError::NotEnoughParameters)?;
        match what {
            "break" => Ok(Command::BreakShow),
            "status" => Ok(Command::StatusShow),
            "mem" => Self::parse_show_mem(params),
            other => Err(ParseError::InvalidParameter(String::from(other))),
        }
        
    }

    fn parse_break<'a, I: Iterator<Item=&'a str>>(mut params: I) -> Result<Command, ParseError> {
        let addr = params.next().ok_or(ParseError::NotEnoughParameters).and_then(Self::parse_addr)?;
        Ok(Command::BreakSet { addr })
    }

    fn parse_delete<'a, I: Iterator<Item=&'a str>>(mut params: I) -> Result<Command, ParseError> {
        match params.next() {
            Some(addr) => Self::parse_addr(addr).map(|a| Command::BreakDelete { addr: Some(a) }),
            None => Ok(Command::BreakDelete { addr: None }),
        }
    }

    fn parse_show_mem<'a, I: Iterator<Item=&'a str>>(mut params: I) -> Result<Command, ParseError> {
        match params.next() {
            Some(addr) => Self::parse_addr(addr).map(|a| Command::MemShow { addr: Some(a) }),
            None => Ok(Command::MemShow { addr: None }),
        }
    }

    fn parse_addr(s: &str) -> Result<Addr, ParseError> {
        if let Some(pa) = s.strip_prefix(":") {
            u32::from_str_radix(pa, 16)
                .map(|a| Addr::Physical(a))
                .or(Err(ParseError::InvalidParameter(String::from(s))))
        } else {
            u16::from_str_radix(s, 16)
                .map(|a| Addr::Logical(a))
                .or(Err(ParseError::InvalidParameter(String::from(s))))
        }
    }
}
