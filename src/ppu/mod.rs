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

use crate::{
    cpu::interrupts::{Flags, InterruptFlags},
    mmu::memory::Memory,
};
use bitflags::bitflags;
use log::warn;
use std::cmp::{min, Ordering};
use std::{cell::RefCell, rc::Rc};
use tinyvec::*;

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

const STAT_UNUSED_MASK: u8 = (1 << 7);

/// Gameboy DMG-01 grey scale colors.
const BLACK: u32 = 0x00000000u32;
const DGRAY: u32 = 0x00555555u32;
const LGRAY: u32 = 0x00AAAAAAu32;
const WHITE: u32 = 0x00FFFFFFu32;

bitflags!(
    /// Sprite Attributes
    #[derive(Default, Debug, Clone, Copy)]
    struct SpriteFlags: u8 {
        const UNUSED = 0b_0000_1111; // CGB Mode Only.
        const PALETTE     = 0b_0001_0000;
        const FLIPX       = 0b_0010_0000;
        const FLIPY       = 0b_0100_0000;
        const PRIORITY    = 0b_1000_0000;
    }
);

/// Sprite (Object)
#[derive(Default, Debug, Clone, Copy)]
struct Sprite {
    /// Y Position
    y: isize,

    /// X Position
    x: isize,

    /// Tile Number
    tile: u8,

    /// Attributes
    attr: SpriteFlags,
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
    #[derive(Default, Debug, Clone, Copy)]
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
    #[derive(Default, Debug, Clone, Copy)]
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
#[derive(Copy, Clone, PartialEq, Eq)]
enum Mode {
    HBlank = 0,
    VBlank = 1,
    AccessOAM = 2,  // OAM Search
    AccessVRAM = 3, // Data Transfer
}

impl Mode {
    fn bits(&self) -> u8 {
        match *self {
            Mode::HBlank => 0,
            Mode::VBlank => 1,
            Mode::AccessOAM => 2,
            Mode::AccessVRAM => 3,
        }
    }
}

/// PPU (Picture Processing Unit)
pub struct Ppu {
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
    //dma: u8,

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
    sprite: [Sprite; 40],
    unmapped_bg: Vec<u8>,

    /* Interrupt Flags from MMU */
    if_: Rc<RefCell<InterruptFlags>>,

    /* Internal State */
    cycles: u32,

    /* Public State */
    pub updated: bool,
    pub buffer: Vec<u32>,
}

fn pixel_to_color(pixel: u8) -> u32 {
    match pixel {
        3 => BLACK,
        2 => DGRAY,
        1 => LGRAY,
        0 => WHITE,
        _ => panic!("Invalid value in u8_to_grayscale"),
    }
}

fn pixel_map_by_palette(palette: u8, pixel: u8) -> u8 {
    match pixel {
        3 => (palette >> 6) & 0x3,
        2 => (palette >> 4) & 0x3,
        1 => (palette >> 2) & 0x3,
        0 => (palette >> 0) & 0x3,
        _ => panic!("Invalid value in u8_from_palette"),
    }
}

impl Ppu {
    pub fn new(if_: Rc<RefCell<InterruptFlags>>) -> Self {
        Self {
            control: LcdcFlags::from_bits(0x91).unwrap(),
            //control: LcdcFlags::empty(),
            stat: StatFlags::empty(),
            mode: Mode::AccessOAM,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            //dma: 0,
            bgp: 0xFC,
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0,
            wx: 0,
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            unmapped_bg: vec![0; SCREEN_PIXELS],
            sprite: [Default::default(); 40],
            if_,
            cycles: ACCESS_OAM_CYCLES,
            updated: false,
            buffer: vec![0; SCREEN_PIXELS],
        }
    }

    /// Handle switching between modes and triggering interrupts
    fn update_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.cycles += match self.mode {
            Mode::AccessOAM => ACCESS_OAM_CYCLES,
            Mode::AccessVRAM => ACCESS_VRAM_CYCLES,
            Mode::HBlank => HBLANK_CYCLES,
            Mode::VBlank => VBLANK_CYCLES,
        };
        match self.mode {
            Mode::HBlank => {
                self.stat.remove(StatFlags::COMPARE);
                self.stat.remove(StatFlags::OAM_INT);
                self.stat.remove(StatFlags::VBLANK_INT);
                if self.stat.contains(StatFlags::HBLANK_INT) {
                    self.if_.borrow_mut().set(Flags::LCDStat);
                }
            }
            Mode::VBlank => {
                self.stat.remove(StatFlags::COMPARE);
                self.stat.remove(StatFlags::HBLANK_INT);
                self.stat.remove(StatFlags::OAM_INT);

                self.if_.borrow_mut().set(Flags::VBlank);
                if self.stat.contains(StatFlags::VBLANK_INT) {
                    self.if_.borrow_mut().set(Flags::LCDStat);
                }

                self.updated = true;
            }
            Mode::AccessOAM => {
                self.stat.remove(StatFlags::HBLANK_INT);
                self.stat.remove(StatFlags::VBLANK_INT);

                if self.stat.contains(StatFlags::OAM_INT) {
                    self.if_.borrow_mut().set(Flags::LCDStat);
                }

                self.check_lyc();
            }
            Mode::AccessVRAM => {
                self.stat.remove(StatFlags::HBLANK_INT);
                self.stat.remove(StatFlags::VBLANK_INT);
                self.stat.remove(StatFlags::OAM_INT);
            }
        }
    }

    /// Check the LYC=LY flag and trigger interrupt if necessary
    fn check_lyc(&mut self) {
        if self.ly == self.lyc {
            self.stat.insert(StatFlags::COMPARE);
            if self.stat.contains(StatFlags::COMPARE_INT) {
                self.if_.borrow_mut().set(Flags::LCDStat);
            }
        } else {
            self.stat.remove(StatFlags::COMPARE);
        }
    }

    #[allow(clippy::eq_op)]
    pub fn get_tile_line(&self, tile_idx: u8, line_idx: usize, is_sprite: bool) -> Vec<u8> {
        assert!(line_idx < 8);
        let line_idx = line_idx as isize;
        let addr = if is_sprite || self.control.contains(LcdcFlags::BG_WINDOW_TILE_DATA) {
            let baseaddr = 0x8000 - 0x8000;
            let tile_idx = tile_idx as isize;
            baseaddr + (tile_idx * 8 + line_idx) * 2
        } else {
            let baseaddr = 0x8800 - 0x8000;
            let tile_idx = (tile_idx as i8) as isize;
            baseaddr + (tile_idx * 8 + line_idx) * 2
        } as usize;

        let byte1 = self.vram[addr];
        let byte2 = self.vram[addr + 1];

        let mut pxs = Vec::with_capacity(8);

        for j in (0..8).rev() {
            let bit1 = (byte1 >> j) & 0x1;
            let bit2 = (byte2 >> j) & 0x1;
            let pixel = bit1 << 1 | bit2;
            pxs.push(pixel);
        }
        pxs
    }

    fn build_background(&mut self) {
        let bg_palette = self.bgp;
        let x = self.scx as usize;
        let y = self.scy as usize;
        let tile_base = if self.control.contains(LcdcFlags::BG_TILE_MAP_ADDRESS) {
            0x9C00
        } else {
            0x9800
        } - 0x8000;

        /*
         * fill the screen from row 0..HEIGHT, col 0..WIDTH
         * the gameboy can set scx and scy so that the left-top corner of the screen
         * starts from (scx, scy)
         */
        for row in 0..SCREEN_HEIGHT {
            let offset_row = (row + y) % 256;
            if offset_row >= SCREEN_HEIGHT {
                break;
            }
            let tile_row = row / 8;
            let line_idx = row % 8;

            for col in 0..(SCREEN_WIDTH / 8) {
                let tile_addr = tile_base + tile_row * 32 + col;
                let tile_idx = self.vram[tile_addr];
                let pixels = self.get_tile_line(tile_idx, line_idx, false);

                let pixel_start = offset_row * SCREEN_WIDTH + col * 8 + x;
                if pixel_start >= (offset_row + 1) * SCREEN_WIDTH {
                    break;
                }
                let pixel_end = min((offset_row + 1) * SCREEN_WIDTH, pixel_start + 8);

                self.unmapped_bg
                    .splice(pixel_start..pixel_end, pixels.iter().cloned());
                self.buffer.splice(
                    pixel_start..pixel_end,
                    pixels
                        .iter()
                        .map(|p| pixel_map_by_palette(bg_palette, *p))
                        .map(|p| pixel_to_color(p)),
                );
            }
        }
    }

    fn build_sprite(&mut self) {
        for sprite in self.sprite.iter() {
            // check sprite intersect with screen
            let sprite_height = if self.control.contains(LcdcFlags::OBJ_SIZE) {
                16
            } else {
                8
            };
            if sprite.y + sprite_height <= 0
                || sprite.x + 8 <= 0
                || (sprite.x as usize) > SCREEN_WIDTH
                || (sprite.y as usize) > SCREEN_HEIGHT
            {
                continue;
            }

            let palette = if sprite.attr.contains(SpriteFlags::PALETTE) {
                self.obp1
            } else {
                self.obp0
            };

            for row_idx in 0..8 {
                let y = sprite.y + row_idx as isize;
                if y < 0 || (y as usize) > SCREEN_HEIGHT {
                    continue;
                }
                let y_idx = if sprite.attr.contains(SpriteFlags::FLIPY) {
                    7 - row_idx
                } else {
                    row_idx
                };
                let pixels = self.get_tile_line(sprite.tile, y_idx, true);
                for col_idx in 0..8 {
                    let x = sprite.x + col_idx as isize;
                    if x < 0 || (x as usize) > SCREEN_WIDTH {
                        continue;
                    }
                    let x_idx = if sprite.attr.contains(SpriteFlags::FLIPX) {
                        7 - col_idx
                    } else {
                        col_idx
                    };
                    if sprite.attr.contains(SpriteFlags::PRIORITY)
                        && self.unmapped_bg[y as usize * SCREEN_WIDTH + x as usize] != 0
                    {
                        continue;
                    }

                    // fill the buffer
                    let dibit = pixel_map_by_palette(palette, pixels[x_idx]);
                    if dibit != 0 {
                        let color = pixel_to_color(dibit);
                        self.buffer[y as usize * SCREEN_WIDTH + x as usize] = color;
                    }
                }
            }
        }
    }

    pub fn build_screen(&mut self) {
        if self.control.contains(LcdcFlags::BG_DISPLAY_ENABLE) {
            self.build_background();
        } else {
            self.unmapped_bg.iter_mut().map(|x| *x = 0).count();
        }

        if self.control.contains(LcdcFlags::OBJ_DISPLAY_ENABLE) {
            self.build_sprite();
        }
    }

    fn update_sprite(&mut self, addr: usize) {
        let sprite_idx = addr / 4;
        let value = self.oam[addr];
        match addr & 0x03 {
            0 => self.sprite[sprite_idx].y = value as isize - 16,
            1 => self.sprite[sprite_idx].x = value as isize - 8,
            2 => self.sprite[sprite_idx].tile = value,
            3 => {
                self.sprite[sprite_idx]
                    .attr
                    .set(SpriteFlags::PRIORITY, ((value >> 0x7) & 0x1) != 0);
                self.sprite[sprite_idx]
                    .attr
                    .set(SpriteFlags::FLIPY, ((value >> 0x6) & 0x1) != 0);
                self.sprite[sprite_idx]
                    .attr
                    .set(SpriteFlags::FLIPX, ((value >> 0x5) & 0x1) != 0);
                self.sprite[sprite_idx]
                    .attr
                    .set(SpriteFlags::PALETTE, ((value >> 0x4) & 0x1) != 0);
            }
            _ => {}
        }
    }
}

impl Memory for Ppu {
    fn read8(&self, addr: u16) -> u8 {
        match addr {
            // Only read if current mode allows
            0x8000..=0x9FFF | 0xFE00..=0xFE9F => match self.mode {
                Mode::AccessOAM => {
                    if (0xFE00..0xFE9F).contains(&addr) {
                        return self.oam[(addr - 0xFE00) as usize];
                    }
                    UNDEFINED_READ
                }
                Mode::AccessVRAM => {
                    if (0x8000..0x9FFF).contains(&addr) {
                        return self.vram[(addr - 0x8000) as usize];
                    }
                    UNDEFINED_READ
                }
                _ => UNDEFINED_READ,
            },
            0xFF40 => self.control.bits(),
            0xFF41 => {
                if !self.control.contains(LcdcFlags::BG_DISPLAY_ENABLE) {
                    STAT_UNUSED_MASK
                } else {
                    self.mode.bits() | self.stat.bits() | STAT_UNUSED_MASK
                }
            }
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            //0xFF46 => self.dma,
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
                    if (0xFE00..0xFE9F).contains(&addr) {
                        self.oam[(addr - 0xFE00) as usize] = val;
                        self.update_sprite((addr - 0xFE00) as usize);
                    }
                }
                Mode::AccessVRAM => {
                    if (0x8000..0x9FFF).contains(&addr) {
                        self.vram[(addr - 0x8000) as usize] = val;
                    }
                }
                _ => (),
            },
            0xFF40 => {
                let new_control = LcdcFlags::from_bits_truncate(val);
                if !new_control.contains(LcdcFlags::BG_DISPLAY_ENABLE)
                    && self.control.contains(LcdcFlags::BG_DISPLAY_ENABLE)
                {
                    if self.mode != Mode::VBlank {
                        panic!("Warning! LCD off, but not in VBlank");
                    }
                    self.ly = 0;
                }
                if new_control.contains(LcdcFlags::BG_DISPLAY_ENABLE)
                    && !self.control.contains(LcdcFlags::BG_DISPLAY_ENABLE)
                {
                    self.mode = Mode::HBlank;
                    self.cycles = ACCESS_OAM_CYCLES;
                    self.stat.insert(StatFlags::COMPARE);
                }
                self.control = new_control;
            }
            0xFF41 => {
                let new_stat = StatFlags::from_bits_truncate(val);
                self.stat = (self.stat & StatFlags::COMPARE)
                    | (new_stat & StatFlags::HBLANK_INT)
                    | (new_stat & StatFlags::VBLANK_INT)
                    | (new_stat & StatFlags::OAM_INT)
                    | (new_stat & StatFlags::COMPARE_INT);
            }
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => self.ly = val,
            0xFF45 => self.lyc = val,
            0xFF46 => {
                /* dma copy 40 * 28 bits data to OAM zone 0xFE00-0xFE9F
                 * each sprite takes 28 bits space (note that 4 bits are not used in each sprite)
                 * the source address can be designated every 0x100 from 0x0000 to 0xF100.
                 * Depend on the value stored to dma IO line:
                 * 0x00 -> 0x0000
                 * 0x01 -> 0x0100
                 * ...
                 */
                let addr = (val as u16) << 8;
                // copy memory to OAM
                for i in 0..(40 * 4) {
                    let byte = self.read8(addr + i);
                    self.write8(0xFE00 + i, byte);
                }
                //self.dma = val;
            }
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

    fn cycle(&mut self, _: u32) -> u32 {
        // Is the LDC on?
        if !self.control.contains(LcdcFlags::LCD_DISPLAY_ENABLE) {
            return 0;
        }

        // STAT mode=0 interrupt happens one cycle before the mode actually changes
        self.cycles -= 1;
        if self.cycles == 1
            && self.mode == Mode::AccessVRAM
            && self.stat.contains(StatFlags::HBLANK_INT)
        {
            self.if_.borrow_mut().set(Flags::LCDStat);
        }
        if self.cycles > 0 {
            return 0;
        }

        // Take action depending on current mode
        match self.mode {
            Mode::AccessOAM => {
                self.update_mode(Mode::AccessVRAM);
            }
            Mode::AccessVRAM => {
                self.build_screen();
                self.update_mode(Mode::HBlank);
            }
            Mode::HBlank => {
                self.ly += 1;
                if self.ly < 144 {
                    self.update_mode(Mode::AccessOAM);
                } else {
                    self.update_mode(Mode::VBlank);
                }
                self.check_lyc();
            }
            Mode::VBlank => {
                self.ly += 1;
                if self.ly > 153 {
                    self.ly = 0;
                    self.update_mode(Mode::AccessOAM);
                } else {
                    self.cycles += VBLANK_CYCLES;
                }
                self.check_lyc();
            }
        }
        self.cycles
    }
}
