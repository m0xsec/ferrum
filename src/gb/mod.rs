use crate::boot;
use crate::cpu;
use crate::mmu;
use crate::mmu::memory::Memory;
use log::info;
use std::cell::RefCell;
use std::rc::Rc;

/// The GameBoy DMG-01 (non-color).
pub struct GameBoy {
    /// The heart of the Gameboy, the CPU.
    /// The CPU is responsible for decoding and executing instructions.
    /// The DMG-01 had a Sharp LR35902 CPU (speculated to be a SM83 core), which is a hybrid of the Z80 and the 8080.
    cpu: cpu::CPU,

    /// The DMG-01 didn't have an actual Memory Management Unit (MMU), but it had a memory-mapped I/O system with a single RAM chip.
    /// To make emulation easier, we will define a MMU.
    /// The MMU is responsible for mapping memory addresses to actual memory locations.
    mmu: Rc<RefCell<mmu::MMU>>,
}

impl GameBoy {
    /// Loads the Gameboy DMG-01 Boot ROM into memory.
    fn read_boot_rom(&mut self) {
        info!("Loading boot rom.");
        for (addr, val) in boot::BOOTROM.iter().enumerate() {
            self.mmu.borrow_mut().write(addr as u16, *val);
        }
    }
}

impl GameBoy {
    /// Initialize Gameboy Hardware
    pub fn power_on() -> Self {
        let mmu = Rc::new(RefCell::new(mmu::MMU::new()));
        let cpu = cpu::CPU::power_on(mmu.clone());
        Self { mmu, cpu }
    }

    /// Loads the Gameboy DMG-01 Boot ROM
    pub fn boot_rom(&mut self) {
        // Read boot ROM into memory
        self.read_boot_rom();

        // NOTE: Testing prohibited memory operation warning log
        self.mmu.borrow_mut().write(0xFEA0, 0x2C);
        self.cpu.test();

        self.cpu.dump_registers();
    }
}
