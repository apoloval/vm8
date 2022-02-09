extern crate vm8;

use rustyline::Editor;
use rustyline::error::ReadlineError;

use vm8::sys::nexus::System;
use vm8::sys::nexus::Command;

fn main() {
    let mut sys = System::new();
    let mut rl = Editor::<()>::new();
    println!("Nexus Computer System Emulator v{}", env!("CARGO_PKG_VERSION"));
    println!("Copyright (C) 2021-2022 Alvaro Polo");
    println!("");
    loop {
        match rl.readline(sys.prompt().as_str()) {
            Ok(line) if line.len() == 0  => {},
            Ok(line) => {
                match Command::parse(line.as_str()) {
                    Ok(Command::Help) => Command::print_help(),
                    Ok(Command::Exit) => break,
                    Ok(cmd) => sys.exec_cmd(cmd),
                    Err(err) => println!("Error: {}", err),
                };
            },
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => {
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            },
        }

    }
}
