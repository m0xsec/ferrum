use log::{info, warn};

mod boot;
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

// TODO: Move testing flag to a command line arg later.
const TESTING: bool = true;

fn main() {
    env_logger::init();
    info!("ferrum is a WIP. Most functionality is not implemented.");

    if TESTING {
        warn!("Testing mode enabled!");
    }

    // TODO: ROM loading, launch the Gameboy emulator threads, etc, etc
    let mut ferrum = gb::GameBoy::power_on();
    ferrum.boot_rom(TESTING);
    warn!("Remaining Gameboy boot process is not yet implemented.");
    ferrum.run(TESTING);
}
