use log::info;

mod boot;
mod cpu;
mod gb;
mod mmu;

fn main() {
    env_logger::init();
    info!("ferrum is a WIP. Most functionality is not implemented.");

    // TODO: ROM loading, launch the Gameboy emulator threads, etc, etc
    let mut ferrum = gb::GameBoy::new();
    ferrum.power_on();
}
