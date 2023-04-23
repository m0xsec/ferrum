use crate::cpu;
use crate::mmu;
use log::warn;
use std::cell::RefCell;
use std::rc::Rc;

/// The GameBoy DMG-01 (non-color).
pub struct GameBoy {
    /// The heart of the Gameboy, the CPU.
    /// The CPU is responsible for decoding and executing instructions.
    /// The DMG-01 had a Sharp LR35902 CPU (speculated to be a SM83 core), which is a hybrid of the Z80 and the 8080.
    cpu: cpu::Cpu,
}
impl GameBoy {
    /// Initialize Gameboy Hardware
    pub fn power_on(rom_path: String) -> Self {
        let mmu = Rc::new(RefCell::new(mmu::Mmu::new(rom_path)));
        let cpu = cpu::Cpu::power_on(mmu);

        Self { cpu }
    }

    /// Run Gameboy emulation
    pub fn run(&mut self) {
        warn!("Emulation loop is a work in progress, no threading or event handling.");

        loop {
            self.cpu.dump_registers();
            self.cpu.cycle();
        }
    }
}
