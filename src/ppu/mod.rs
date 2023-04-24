/*
PPU References:
https://youtu.be/HyzD8pNlpwI?t=1765 (Ultimate Game Boy Talk)
https://gbdev.io/pandocs/Rendering.html
https://gbdev.io/pandocs/STAT.html
https://gbdev.io/pandocs/Scrolling.html
https://pixelbits.16-b.it/GBEDG/ppu/#the-screen
https://emudev.de/gameboy-emulator/%e2%af%88-ppu-rgb-arrays-and-sdl/
https://github.com/Gekkio/mooneye-gb
 */

use crate::{cpu::interrupts::InterruptFlags, mmu::memory::Memory};
use bitflags::bitflags;
use log::warn;
use std::{cell::RefCell, rc::Rc};

/// The Gameboy outputs a 160x144 pixel LCD screen.
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

/// The PPU had varying cycles depending on the mode it was in.
const ACCESS_OAM_CYCLES: u32 = 21;
const ACCESS_VRAM_CYCLES: u32 = 43;
const HBLANK_CYCLES: u32 = 50;
const VBLANK_CYCLES: u32 = 114;

/// The PPU always returned 0xFF for undefined reads.
const UNDEFINED_READ: u8 = 0xFF;

bitflags!(
    /// Sprite Attributes
    struct SpriteFlags: u8 {
        const UNUSED = 0b_0000_1111; // CGB Mode Only.
        const PALETTE     = 0b_0001_0000;
        const FLIPX       = 0b_0010_0000;
        const FLIPY       = 0b_0100_0000;
        const PRIORITY    = 0b_1000_0000;
    }
);

/// Sprite (Object)
struct Sprite {
    /// Y Position
    y: u8,

    /// X Position
    x: u8,

    /// Tile Number
    tile: u8,

    /// Attributes
    attr: SpriteFlags,
}

/// Gameboy (DMG-01) had 4 shades of gray for possible colors.
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
enum Color {
    Off = 0,
    Light = 1,
    Dark = 2,
    On = 3,
}

impl Color {
    fn from_u8(value: u8) -> Color {
        match value {
            1 => Color::Light,
            2 => Color::Dark,
            3 => Color::On,
            _ => Color::Off,
        }
    }
}

impl From<Color> for u8 {
    fn from(val: Color) -> Self {
        match val {
            Color::Off => 0,
            Color::Light => 1,
            Color::Dark => 2,
            Color::On => 3,
        }
    }
}

/// Color Palettes could be defined for OBJs and BGs.
#[derive(Clone)]
struct Palette {
    off: Color,
    light: Color,
    dark: Color,
    on: Color,
    /// Bit 7-6 - Color for index 3
    /// Bit 5-4 - Color for index 2
    /// Bit 3-2 - Color for index 1
    /// Bit 1-0 - Color for index 0
    bits: u8,
}

impl Palette {
    fn new() -> Palette {
        Self {
            off: Color::On,
            light: Color::On,
            dark: Color::On,
            on: Color::On,
            bits: 0xff,
        }
    }
    fn get(&self, color: &Color) -> Color {
        match *color {
            Color::Off => self.off,
            Color::Light => self.light,
            Color::Dark => self.dark,
            Color::On => self.on,
        }
    }
    fn set_bits(&mut self, value: u8) {
        self.off = Color::from_u8(value & 0x3);
        self.light = Color::from_u8((value >> 2) & 0x3);
        self.dark = Color::from_u8((value >> 4) & 0x3);
        self.on = Color::from_u8((value >> 6) & 0x3);
        self.bits = value;
    }
}

bitflags!(
    /// LCDC (LCD Control) Register
    /// FF40 — LCDC: LCD Control
    /// Bit 7 - LCD Display Enable             (0=Off, 1=On)
    /// Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
    /// Bit 5 - Window Display Enable          (0=Off, 1=On)
    /// Bit 4 - BG & Window Tile Data Select   (0=8800-97FF, 1=8000-8FFF)
    /// Bit 3 - BG Tile Map Display Select     (0=9800-9BFF, 1=9C00-9FFF)
    /// Bit 2 - OBJ (Sprite) Size              (0=8x8, 1=8x16)
    /// Bit 1 - OBJ (Sprite) Display Enable    (0=Off, 1=On)
    /// Bit 0 - BG Display (for CGB see below) (0=Off, 1=On)
    struct LcdcFlags: u8 {
        const LCD_DISPLAY_ENABLE = 1 << 7;
        const WINDOW_TILE_MAP_ADDRESS = 1 << 6;
        const WINDOW_DISPLAY_ENABLE = 1 << 5;
        const BG_WINDOW_TILE_DATA = 1 << 4;
        const BG_TILE_MAP_ADDRESS = 1 << 3;
        const OBJ_SIZE = 1 << 2;
        const OBJ_DISPLAY_ENABLE = 1 << 1;
        const BG_DISPLAY_ENABLE = 1 << 0;
    }
);

bitflags!(
    /// STAT (LCDC Status) Register
    /// FF41 — STAT: LCD status
    /// Bit 6 - LYC=LY Compare Interrupt (1=Enable) (Read/Write)
    /// Bit 5 - Mode 2 OAM Interrupt         (1=Enable) (Read/Write)
    /// Bit 4 - Mode 1 V-Blank Interrupt     (1=Enable) (Read/Write)
    /// Bit 3 - Mode 0 H-Blank Interrupt     (1=Enable) (Read/Write)
    /// Bit 2 - Compare Flag  (0:LYC<>LY, 1:LYC=LY) (Read Only)
    /// Bit 0-1 - Mode Flag       (Mode 0-3, see below) (Read Only)
    ///   0: During H-Blank
    ///   1: During V-Blank
    ///   2: During Searching OAM-RAM
    ///   3: During Transferring Data to LCD Driver
    struct StatFlags: u8 {
        const COMPARE_INT= 1 << 6;
        const OAM_INT = 1 << 5;
        const VBLANK_INT = 1 << 4;
        const HBLANK_INT = 1 << 3;
        const COMPARE = 1 << 2;
    }
);

/// STAT (LCDC Status) Register Mode Flag
/// Bit 0-1 - Mode Flag       (Mode 0-3, see below) (Read Only)
///   0: During H-Blank
///   1: During V-Blank
///   2: During Searching OAM-RAM
///   3: During Transferring Data to LCD Driver
enum Mode {
    HBlank = 0,
    VBlank = 1,
    AccessOAM = 2,  // OAM Search
    AccessVRAM = 3, // Data Transfer
}

/// PPU (Picture Processing Unit)
struct Ppu {
    /* Registers */
    /// 0xFF40 - LCDC (LCD Control) Register (R/W)
    control: LcdcFlags,

    /// 0xFF41 - STAT (LCDC Status) Register (R/W)
    stat: StatFlags,
    mode: Mode,

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

impl Ppu {
    pub fn new(if_: Rc<RefCell<InterruptFlags>>) -> Self {
        Self {
            control: LcdcFlags::empty(),
            stat: StatFlags::empty(),
            mode: Mode::AccessOAM,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            dma: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            wy: 0,
            wx: 0,
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            if_,
        }
    }
}

impl Memory for Ppu {
    fn read8(&self, addr: u16) -> u8 {
        match addr {
            // Only read if current mode allows
            0x8000..=0x9FFF | 0xFE00..=0xFE9F => match self.mode {
                Mode::AccessOAM => {
                    if (0xFE00..0xFEA0).contains(&addr) {
                        return self.oam[(addr - 0xFE00) as usize];
                    }
                    UNDEFINED_READ
                }
                Mode::AccessVRAM => {
                    if (0x8000..0xA000).contains(&addr) {
                        return self.vram[(addr - 0x8000) as usize];
                    }
                    UNDEFINED_READ
                }
                _ => UNDEFINED_READ,
            },
            0xFF40 => self.control.bits(),
            0xFF41 => self.stat.bits(),
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF46 => self.dma,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => UNDEFINED_READ,
        }
    }

    fn write8(&mut self, addr: u16, val: u8) {
        match addr {
            // Only write if current mode allows
            0x8000..=0x9FFF | 0xFE00..=0xFE9F => match self.mode {
                Mode::AccessOAM => {
                    if (0xFE00..0xFEA0).contains(&addr) {
                        self.oam[(addr - 0xFE00) as usize] = val;
                    }
                }
                Mode::AccessVRAM => {
                    if (0x8000..0xA000).contains(&addr) {
                        self.vram[(addr - 0x8000) as usize] = val;
                    }
                }
                _ => (),
            },
            0xFF40 => self.control = LcdcFlags::from_bits_truncate(val),
            0xFF41 => self.stat = StatFlags::from_bits_truncate(val),
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => self.ly = val,
            0xFF45 => self.lyc = val,
            0xFF46 => self.dma = val,
            0xFF47 => self.bgp = val,
            0xFF48 => self.obp0 = val,
            0xFF49 => self.obp1 = val,
            0xFF4A => self.wy = val,
            0xFF4B => self.wx = val,
            _ => warn!("Invalid write to PPU: {:#06x} <- {:#04x}", addr, val),
        }
    }

    fn read16(&self, addr: u16) -> u16 {
        u16::from(self.read8(addr)) | (u16::from(self.read8(addr + 1)) << 8)
    }

    fn write16(&mut self, addr: u16, val: u16) {
        self.write8(addr, (val & 0xFF) as u8);
        self.write8(addr + 1, (val >> 8) as u8);
    }

    fn cycle(&mut self, ticks: u32) -> u32 {
        todo!()
    }
}
