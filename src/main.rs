use clap::{Arg, Command};
use log::{info, warn};

mod boot;
mod cartridge;
mod cpu;
mod gb;
mod mmu;

#[macro_use]
extern crate lazy_static;

// TODO: For graphics and input, use minifb, or Bevy, or something else?
// https://github.com/emoon/rust_minifb
// https://bevyengine.org/
// Example using minifb for graphics: https://github.com/YushiOMOTE/rgy/blob/master/core/examples/pc/hardware.rs#L75
// Can just dump GB VRAM to the minifb window buffer :O
// Bevy would give more flexibility, but getting the ECS to play nice with the Gameboy's architecture might be tricky...

fn main() {
    env_logger::init();
    info!("ferrum is a WIP. Most functionality is not implemented.");

    let matches = Command::new("ferrum")
        .version("0.1.0")
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

    // TODO: ROM loading, launch the Gameboy emulator threads, etc, etc
    let mut ferrum = gb::GameBoy::power_on(rom_path.to_string());
    ferrum.boot();
    warn!("Remaining Gameboy boot process is not yet implemented.");
    ferrum.run();
}
