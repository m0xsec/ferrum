use crate::cartridge::Cartridge;
use crate::cpu;
use crate::mmu;
use crate::mmu::memory::Memory;
use log::{info, warn};
use std::cell::RefCell;
use std::rc::Rc;

/// The GameBoy DMG-01 (non-color).
pub struct GameBoy {
    /// ROM file path, provided as a command line argument.
    rom_path: String,

    /// Gameboy Cartridge (ROM).
    cartridge: Cartridge,

    /// The heart of the Gameboy, the CPU.
    /// The CPU is responsible for decoding and executing instructions.
    /// The DMG-01 had a Sharp LR35902 CPU (speculated to be a SM83 core), which is a hybrid of the Z80 and the 8080.
    cpu: cpu::Cpu,

    /// The DMG-01 didn't have an actual Memory Management Unit (MMU), but it had a memory-mapped I/O system with a single RAM chip.
    /// To make emulation easier, we will define a MMU.
    /// The MMU is responsible for mapping memory addresses to actual memory locations.
    mmu: Rc<RefCell<mmu::Mmu>>,
}

impl GameBoy {
    /// Loads a test ROM into memory from a file.
    /// NOTE: We will have a similar function for loading a ROM from a file later on.
    fn read_test_rom(&mut self, path: &str) {
        info!("Loading test rom, {}.", path);
        let rom = std::fs::read(path).unwrap();
        for (addr, val) in rom.iter().enumerate() {
            self.mmu.borrow_mut().write8(addr as u16, *val);
        }
    }
}

impl GameBoy {
    /// Initialize Gameboy Hardware
    pub fn power_on(rom_path: String) -> Self {
        let mmu = Rc::new(RefCell::new(mmu::Mmu::new()));
        let cpu = cpu::Cpu::power_on(mmu.clone());
        let cartridge = Cartridge::new(rom_path.clone());
        Self {
            rom_path,
            cartridge,
            mmu,
            cpu,
        }
    }

    /// Boots the Gameboy DMG-01
    pub fn boot(&mut self) {
        // NOTE: No need to load BOOT ROM directly, MMU will handle reading from the BOOT ROM before it is unmapped.
        // TODO: Utilize cartridge data and MBC stuff later...
        self.read_test_rom(&self.rom_path.clone());
        self.cpu.dump_registers();
    }

    /// Run Gameboy emulation
    pub fn run(&mut self) {
        warn!("Emulation loop is a work in progress, no threading or event handling.");

        println!("\nCartridge Info:");
        println!("\tCartridge Title: {}", self.cartridge.title());
        println!("\tCartridge Type: {:?}", self.cartridge.mbc());
        println!("\tROM Size: {:?}", self.cartridge.rom_size());
        println!("\tRAM Size: {:?}", self.cartridge.ram_size());
        println!(
            "\tDestination Code: {:?}",
            self.cartridge.destination_code()
        );
        println!(
            "\tNew Licensee Code: {:?}",
            self.cartridge.new_licensee_code()
        );
        println!(
            "\tOld Licensee Code: {:?}\n",
            self.cartridge.old_licensee_code()
        );

        loop {
            self.cpu.dump_registers();
            self.cpu.cycle();
        }
    }
}
