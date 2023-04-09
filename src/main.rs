use log::{info, warn};

mod boot;
mod cpu;
mod gb;
mod mmu;

fn main() {
    env_logger::init();
    info!("ferrum is a WIP. Most functionality is not implemented.");

    // TODO: ROM loading, launch the Gameboy emulator threads, etc, etc
    let mut ferrum = gb::GameBoy::power_on();
    ferrum.boot_rom();
    warn!("Remaining Gameboy boot process is not yet implemented.");
}
