use super::Cartridge;
use crate::mmu::memory::Memory;

/// No MBC (ROM Only) - https://gbdev.io/pandocs/nombc.html
/// Small games of not more than 32 KiB ROM do not require a MBC chip for ROM banking.
/// The ROM is directly mapped to memory at $0000-7FFF.
/// Optionally up to 8 KiB of RAM could be connected at $A000-BFFF, using a discrete logic decoder in place of a full MBC chip.
pub struct RomOnly {
    rom: Vec<u8>,
}

impl RomOnly {
    pub fn new(rom: Vec<u8>) -> Self {
        Self { rom }
    }
}

impl Memory for RomOnly {
    fn read8(&self, addr: u16) -> u8 {
        self.rom[addr as usize]
    }

    fn write8(&mut self, _: u16, _: u8) {}

    fn read16(&self, addr: u16) -> u16 {
        u16::from(self.read8(addr)) | (u16::from(self.read8(addr + 1)) << 8)
    }

    fn write16(&mut self, _: u16, _: u16) {}
}

impl Cartridge for RomOnly {}
