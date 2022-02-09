pub enum Command {
    Help,
    Regs,
    Step,
    MemRead { addr: Option<u16> },
}

impl Command {
    pub fn parse(line: &str) -> Option<Command> {
        let mut params = line.split_whitespace();
        match params.next() {
            Some("help") | Some("?") => Some(Command::Help),
            Some("regs") => Some(Command::Regs),
            Some("step") => Some(Command::Step),
            Some("memread") => Some(Command::MemRead { addr: params.next().and_then(Self::parse_addr) }),
            _ => None,
        }
    }

    pub fn print_help() {
        println!("  help | ?            Print this help");
        println!("  regs                Print status of CPU registers");
        println!("  step                Execute one CPU step");
        println!("  memread [<addr>]    Print memory content starting at <addr>");
        println!("                      (Default address is PC)");
    }

    fn parse_addr(s: &str) -> Option<u16> {
        u16::from_str_radix(s, 16).ok()
    }
}
