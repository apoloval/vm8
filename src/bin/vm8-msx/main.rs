use std::fs::File;
use clap::{App, Arg};

fn main() {
  let matches = App::new("vm8-msx")
    .version("0.1")
    .author("Alvaro Polo <apoloval@gmail.com>")
    .about("An 8-bit emulator for MSX systems based on VM8 library")
    .arg(
      Arg::with_name("model")
                .short("m")
                .long("model")
                .value_name("MODEL")
                .help("The MSX model to be used")
                .takes_value(true),
    )
    .arg(
      Arg::with_name("list")
                .short("l")
                .long("list")
                .help("List the available MSX models")
    )
    .get_matches();
  if matches.is_present("list") {
    println!("svi728     Spectravideo SVI-728 (default)");
    return
  }

  match matches.value_of("model") {
    Some("svi728") | None => {
      let mut bios = File::open("rom/svi728/bios.rom").unwrap();
      let mut msx = vm8::system::msx::svi_728(&mut bios).unwrap();
      let mut scheduler = vm8::emu::Scheduler::new();
      msx.run(&mut scheduler);
    },
    Some(other) => {
      println!("Error: invalid MSX model '{}'", other);
      println!("Use --list flag to enumerate the available MSX models");
    },
  }
}
