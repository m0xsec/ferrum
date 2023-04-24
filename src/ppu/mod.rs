/*
PPU References:
https://youtu.be/HyzD8pNlpwI?t=1765 (Ultimate Game Boy Talk)
https://gbdev.io/pandocs/Rendering.html
https://gbdev.io/pandocs/STAT.html
https://gbdev.io/pandocs/Scrolling.html
https://pixelbits.16-b.it/GBEDG/ppu/#the-screen
https://emudev.de/gameboy-emulator/%e2%af%88-ppu-rgb-arrays-and-sdl/
 */

use std::{cell::RefCell, rc::Rc};

use crate::{cpu::interrupts::InterruptFlags, mmu::memory::Memory};

/// The Gameboy outputs a 160x144 pixel LCD screen.
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

/// Gameboy (DMG-01) had 4 shades of gray for possible colors.
enum Color {
    White,
    LightGray,
    DarkGray,
    Black,
}

/// LCDC (LCD Control) Register
/// FF40 — LCDC: LCD Control
struct Lcdc {
    /// LCD and PPU Enable
    /// 0: Off, 1: On
    lcd_display_enable: u8,

    /// Window Tile Map Address Area
    /// 0: 9800-9BFF, 1: 9C00-9FFF
    window_tile_map_address: u8,

    /// Window Display Enable
    /// 0: Off, 1: On
    window_display_enable: u8,

    /// BG & Window Tile Data Area
    /// 0: 8800-97FF, 1: 8000-8FFF
    bg_window_tile_data: u8,

    /// BG Tile Map Address Area
    /// 0: 9800-9BFF, 1: 9C00-9FFF
    bg_tile_map_address: u8,

    /// OBJ Size
    /// 0: 8x8, 1: 8x16
    obj_size: u8,

    /// OBJ Display Enable
    /// 0: Off, 1: On
    obj_display_enable: u8,

    /// BG & Window Display Enable
    /// 0: Off, 1: On
    bg_display_enable: u8,
}

impl Lcdc {
    fn new() -> Self {
        Self {
            lcd_display_enable: 0x00,
            window_tile_map_address: 0x00,
            window_display_enable: 0x00,
            bg_window_tile_data: 0x00,
            bg_tile_map_address: 0x00,
            obj_size: 0x00,
            obj_display_enable: 0x00,
            bg_display_enable: 0x00,
        }
    }
}

/// STAT Mode Flag
enum Mode {
    HBlank = 0,
    VBlank = 1,
    OamSearch = 2,    // AccessOAM
    DataTransfer = 3, // AccessVRAM
}

/// STAT (LCDC Status) Register
/// FF41 — STAT: LCD status
struct Stat {
    /// LYC=LY Interrupt (1=Enable) (Read/Write)
    lyc_interrupt: u8,

    /// Mode 2 OAM Interrupt         (1=Enable) (Read/Write)
    mode2_interrupt: u8,

    /// Mode 1 V-Blank Interrupt     (1=Enable) (Read/Write)
    mode1_interrupt: u8,

    /// Mode 0 H-Blank Interrupt     (1=Enable) (Read/Write)
    mode0_interrupt: u8,

    /// LYC Flag  (0:LYC<>LY, 1:LYC=LY) (Read Only)
    lyc_flag: u8,

    /// Mode Flag       (Mode 0-3, see below) (Read Only)
    ///   0: During H-Blank
    ///   1: During V-Blank
    ///   2: During Searching OAM-RAM
    ///   3: During Transferring Data to LCD Driver
    mode_flag: u8,
}

impl Stat {
    fn new() -> Self {
        Self {
            lyc_interrupt: 0x00,
            mode2_interrupt: 0x00,
            mode1_interrupt: 0x00,
            mode0_interrupt: 0x00,
            lyc_flag: 0x00,
            mode_flag: 0x00,
        }
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode_flag = mode as u8;
    }

    fn get_mode(&self) -> Mode {
        match self.mode_flag {
            0 => Mode::HBlank,
            1 => Mode::VBlank,
            2 => Mode::OamSearch,
            3 => Mode::DataTransfer,
            _ => panic!("Invalid mode flag"),
        }
    }
}

/// PPU (Picture Processing Unit)
struct Ppu {
    /* Registers */
    /// 0xFF40 - LCDC (LCD Control) Register (R/W)
    ldcd: Lcdc,

    /// 0xFF41 - STAT (LCDC Status) Register (R/W)
    stat: Stat,

    /// 0xFF42 - SCY (Scroll Y) Register (R/W)
    scy: u8,

    /// 0xFF43 - SCX (Scroll X) Register (R/W)
    scx: u8,

    /// 0xFF44 - LY (LCDC Y-Coordinate) Register (R)
    ly: u8,

    /// 0xFF45 - LYC (LY Compare) Register (R/W)
    lyc: u8,

    /// 0xFF46 - DMA (DMA Transfer and Start Address) Register (W)
    dma: u8,

    /// 0xFF47 - BGP (BG Palette Data) Register (R/W)
    bgp: u8,

    /// 0xFF48 - OBP0 (Object Palette 0 Data) Register (R/W)
    obp0: u8,

    /// 0xFF49 - OBP1 (Object Palette 1 Data) Register (R/W)
    obp1: u8,

    /// 0xFF4A - WY (Window Y Position) Register (R/W)
    wy: u8,

    /// 0xFF4B - WX (Window X Position) Register (R/W)
    wx: u8,

    /* Memory */
    vram: [u8; 0x2000], // 8KB Video RAM
    oam: [u8; 0xA0],    // 160B Object Attribute Memory

    /* Interrupt Flags from MMU */
    if_: Rc<RefCell<InterruptFlags>>,
}

impl Memory for Ppu {
    fn read8(&self, addr: u16) -> u8 {
        todo!()
    }

    fn write8(&mut self, addr: u16, val: u8) {
        todo!()
    }

    fn read16(&self, addr: u16) -> u16 {
        todo!()
    }

    fn write16(&mut self, addr: u16, val: u16) {
        todo!()
    }

    fn cycle(&mut self, ticks: u32) -> u32 {
        todo!()
    }
}
