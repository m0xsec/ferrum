use log::{info, warn};

mod boot;
mod cpu;
mod gb;
mod mmu;

#[macro_use]
extern crate lazy_static;

// TODO: For graphics and input, use minifb, or Bevy, or something else?
// https://github.com/emoon/rust_minifb

fn main() {
    env_logger::init();
    info!("ferrum is a WIP. Most functionality is not implemented.");

    // TODO: ROM loading, launch the Gameboy emulator threads, etc, etc
    let mut ferrum = gb::GameBoy::power_on();
    ferrum.boot_rom();
    warn!("Remaining Gameboy boot process is not yet implemented.");
    ferrum.run();
}
