use crate::boot;
use crate::cpu;
use crate::mmu;
use log::info;
use log::warn;

/// The GameBoy DMG-01 (non-color).
pub struct GameBoy {
    // TODO: Implement CPU
    /// The heart of the Gameboy, the CPU.
    /// The CPU is responsible for decoding and executing instructions.
    /// The DMG-01 had a Sharp LR35902 CPU (speculated to be a SM83 core), which is a hybrid of the Z80 and the 8080.
    cpu: cpu::CPU,

    // TODO: Implement MMU
    /// The DMG-01 didn't have an actual Memory Management Unit (MMU), but it had a memory-mapped I/O system with a single RAM chip.
    /// To make emulation easier, we will define a MMU.
    /// The MMU is responsible for mapping memory addresses to actual memory locations.
    mmu: mmu::MMU,
}

impl GameBoy {
    fn load_boot_rom(&mut self) {
        info!("Loading boot rom.");
        for (addr, val) in boot::BOOTROM.iter().enumerate() {
            warn!("Writing to Gameboy memory is not yet implemented.");
            info!("[{:#02x}] --> {:#02x}", addr, val);
        }
    }
}

impl GameBoy {
    pub fn new() -> Self {
        Self {
            cpu: cpu::CPU::new(),
            mmu: mmu::MMU {},
        }
    }

    pub fn power_on(&mut self) {
        warn!("power_on is not fully implemented.");

        // Load boot ROM
        self.load_boot_rom();
    }
}
