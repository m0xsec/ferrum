use clap::{Arg, ArgAction, Command};
use log::{info, warn};

mod boot;
mod cartridge;
mod cpu;
mod gb;
mod mmu;
mod ppu;
mod timer;

#[macro_use]
extern crate lazy_static;

fn main() {
    env_logger::init();
    info!("ferrum is a WIP. Most functionality is not implemented.");

    let matches = Command::new("ferrum")
        .version("0.1.1")
        .author("m0x <https://github.com/m0xsec/ferrum>")
        .about("A Gameboy emulator written in Rust.")
        .arg(
            Arg::new("rom")
                .short('r')
                .long("rom")
                .value_name("FILE")
                .help("Sets the ROM file to load.")
                .required(false),
        )
        .arg(
            Arg::new("headless")
                .long("headless")
                .help("Runs the emulator without a GUI.")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let rom_path = matches.get_one::<String>("rom").map(|s| s.as_str()).unwrap_or("");
    let headless = matches.get_flag("headless");
    let mut ferrum = gb::GameBoy::power_on(rom_path.to_string(), headless);
    ferrum.run();
}
