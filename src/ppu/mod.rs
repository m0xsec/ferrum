use crate::mmu::memory::Memory;

/// The Gameboy outputs a 160x144 pixel LCD screen.
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;
pub const SCREEN_PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

/// The PPU had varying cycles depending on the mode it was in.
const ACCESS_OAM_CYCLES: u32 = 21;
const ACCESS_VRAM_CYCLES: u32 = 43;
const HBLANK_CYCLES: u32 = 50;
const VBLANK_CYCLES: u32 = 114;

/// The PPU always returned 0xFF for undefined reads.
const UNDEFINED_READ: u8 = 0xFF;

/// Gameboy DMG-01 grey scale colors.
const BLACK: u32 = 0x00000000u32;
const DGRAY: u32 = 0x00555555u32;
const LGRAY: u32 = 0x00AAAAAAu32;
const WHITE: u32 = 0x00FFFFFFu32;

/// PPU (Picture Processing Unit)
pub struct Ppu {
    pub buffer: Vec<u32>,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            buffer: vec![BLACK; SCREEN_PIXELS],
        }
    }
}

impl Memory for Ppu {
    fn read8(&self, addr: u16) -> u8 {
        0xFF
    }

    fn write8(&mut self, addr: u16, val: u8) {}

    fn read16(&self, addr: u16) -> u16 {
        u16::from(self.read8(addr)) | (u16::from(self.read8(addr + 1)) << 8)
    }

    fn write16(&mut self, addr: u16, val: u16) {
        self.write8(addr, (val & 0xFF) as u8);
        self.write8(addr + 1, (val >> 8) as u8);
    }

    fn cycle(&mut self, _: u32) -> u32 {
        todo!("PPU is a WIP, plz try again soon <3");
    }
}
