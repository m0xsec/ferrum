use clap::{Arg, Command};
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
        .version("0.1.2 PPU Re-Write")
        .author("m0x <https://github.com/m0xsec/ferrum>")
        .about("A Gameboy emulator written in Rust.")
        .arg(
            Arg::new("rom")
                .short('r')
                .long("rom")
                .value_name("FILE")
                .help("Sets the ROM file to load.")
                .required(true),
        )
        .arg_required_else_help(true)
        .get_matches();

    let rom_path = matches.get_one::<String>("rom").unwrap();
    let mut ferrum = gb::GameBoy::power_on(rom_path.to_string());
    warn!("Graphics, input, and sound are not implemented yet. Ferrum will run, but you won't see anything outside of the console.");
    ferrum.run();
}
