extern crate vm8;

use rustyline::Editor;
use rustyline::error::ReadlineError;

use vm8::sys::nexus::System;
use vm8::sys::nexus::Command;

fn main() {
    let mut sys = System::new();
    let mut rl = Editor::<()>::new();
    println!("Nexus Computer System Emulator");
    println!("Copyright (C) 2021 Alvaro Polo");
    println!("");
    loop {
        match rl.readline(sys.prompt().as_str()) {
            Ok(line) => {
                if let Some(cmd) = Command::parse(line.as_str()) {
                    sys.exec_cmd(cmd);
                } else {
                    println!("Error: unknown command '{}'", line)
                }
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
