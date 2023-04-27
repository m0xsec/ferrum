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
use std::cmp::Ordering;
use std::{cell::RefCell, rc::Rc};
use tinyvec::*;

/// The Gameboy outputs a 160x144 pixel LCD screen.
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;
pub const SCREEN_PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT;
pub type ScreenBuffer = [Color; SCREEN_PIXELS];
pub const SCREEN_EMPTY: ScreenBuffer = [Color::Off; SCREEN_PIXELS];

/// The PPU had varying cycles depending on the mode it was in.
const ACCESS_OAM_CYCLES: u32 = 21;
const ACCESS_VRAM_CYCLES: u32 = 43;
const HBLANK_CYCLES: u32 = 50;
const VBLANK_CYCLES: u32 = 114;

/// The PPU always returned 0xFF for undefined reads.
const UNDEFINED_READ: u8 = 0xFF;

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
pub enum Color {
    On = 0,
    Light = 1,
    Dark = 2,
    Off = 3,
}

impl Color {
    fn from_u8(value: u8) -> Color {
        match value {
            0 => Color::On,
            1 => Color::Light,
            2 => Color::Dark,
            _ => Color::Off,
        }
    }
}

impl From<Color> for u8 {
    fn from(val: Color) -> Self {
        match val {
            Color::On => 0,
            Color::Light => 1,
            Color::Dark => 2,
            Color::Off => 3,
        }
    }
}

pub fn pixel_to_color(pixel: u8) -> u32 {
    match pixel {
        3 => BLACK,
        2 => DGRAY,
        1 => LGRAY,
        0 => WHITE,
        _ => panic!("Invalid value in u8_to_grayscale"),
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
        self.on = Color::from_u8((value >> 0) & 0x3);
        self.light = Color::from_u8((value >> 2) & 0x3);
        self.dark = Color::from_u8((value >> 4) & 0x3);
        self.off = Color::from_u8((value >> 6) & 0x3);
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
#[derive(Copy, Clone, PartialEq, Eq)]
enum Mode {
    HBlank = 0,
    VBlank = 1,
    AccessOAM = 2,  // OAM Search
    AccessVRAM = 3, // Data Transfer
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
    dma: u8,

    /// 0xFF47 - BGP (BG Palette Data) Register (R/W)
    bgp: Palette,

    /// 0xFF48 - OBP0 (Object Palette 0 Data) Register (R/W)
    obp0: Palette,

    /// 0xFF49 - OBP1 (Object Palette 1 Data) Register (R/W)
    obp1: Palette,

    /// 0xFF4A - WY (Window Y Position) Register (R/W)
    wy: u8,

    /// 0xFF4B - WX (Window X Position) Register (R/W)
    wx: u8,

    /* Memory */
    vram: [u8; 0x2000], // 8KB Video RAM
    oam: [u8; 0xA0],    // 160B Object Attribute Memory

    /* Interrupt Flags from MMU */
    if_: Rc<RefCell<InterruptFlags>>,

    /* Internal State */
    cycles: u32,

    /* Public State */
    pub updated: bool,
    pub buffer: Box<ScreenBuffer>,
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
            bgp: Palette::new(),
            obp0: Palette::new(),
            obp1: Palette::new(),
            wy: 0,
            wx: 0,
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            if_,
            cycles: ACCESS_OAM_CYCLES,
            updated: true,
            buffer: Box::new(SCREEN_EMPTY),
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
            Mode::HBlank => {}
            Mode::VBlank => {
                self.if_.borrow_mut().set(Flags::VBlank);
                if self.stat.contains(StatFlags::VBLANK_INT) {
                    self.if_.borrow_mut().set(Flags::LCDStat);
                }
            }
            Mode::AccessOAM => {
                if self.stat.contains(StatFlags::OAM_INT) {
                    self.if_.borrow_mut().set(Flags::LCDStat);
                }
            }
            Mode::AccessVRAM => {}
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

    /// Draw the current line to the video buffer
    fn draw_line(&mut self) {
        let slice_start = SCREEN_WIDTH * self.ly as usize;
        let slice_end = SCREEN_WIDTH + slice_start;
        let pixels = &mut self.buffer[slice_start..slice_end];
        let mut bg_prio = [false; SCREEN_WIDTH];

        if self.control.contains(LcdcFlags::BG_DISPLAY_ENABLE) {
            let map_mask = if self.control.contains(LcdcFlags::BG_TILE_MAP_ADDRESS) {
                0x1c00
            } else {
                0x1800
            };

            let y = self.ly.wrapping_add(self.scy);
            let row = (y / 8) as usize;
            for i in 0..SCREEN_WIDTH {
                let x = (i as u8).wrapping_add(self.scx);
                let col = (x / 8) as usize;

                let tile_num = self.vram[(((row * 32 + col) | map_mask) & 0x1fff)] as usize;
                let tile_num = if self.control.contains(LcdcFlags::BG_WINDOW_TILE_DATA) {
                    tile_num as usize
                } else {
                    128 + ((tile_num as i8 as i16) + 128) as usize
                };

                let line = ((y % 8) * 2) as usize;
                let tile_mask = tile_num << 4;
                let data1 = self.vram[(tile_mask | line) & 0x1fff];
                let data2 = self.vram[(tile_mask | (line + 1)) & 0x1fff];

                let bit = (x % 8).wrapping_sub(7).wrapping_mul(0xff) as usize;

                //let color_l = (data2 << bit) >> 7;
                //let color_h = (data1 << bit) >> 6;
                //let color_value = color_h | color_l;
                //let color_value = ((data2 >> bit) & 1) << 1 | ((data1 >> bit) & 1);
                let color_value =
                    ((data2 >> (7 - (x % 8))) & 1) << 1 | ((data1 >> (7 - (x % 8))) & 1);

                let raw_color = Color::from_u8(color_value);

                let color = self.bgp.get(&raw_color);
                bg_prio[i] = raw_color != Color::Off;
                pixels[i] = color;
                if color_value != 0 {
                    println!("{} {}", i, color_value)
                };
            }
        }
        if self.control.contains(LcdcFlags::WINDOW_DISPLAY_ENABLE) && self.wy <= self.ly {
            let map_mask = if self.control.contains(LcdcFlags::WINDOW_TILE_MAP_ADDRESS) {
                0x1c00
            } else {
                0x1800
            };
            let window_x = self.wx.wrapping_sub(7);

            let y = self.ly - self.wy;
            let row = (y / 8) as usize;
            for i in (window_x as usize)..SCREEN_WIDTH {
                let mut x = (i as u8).wrapping_add(self.scx);
                if x >= window_x {
                    x = i as u8 - window_x;
                }
                let col = (x / 8) as usize;

                let tile_num = self.vram[(((row * 32 + col) | map_mask) & 0x1fff)] as usize;
                let tile_num = if self.control.contains(LcdcFlags::BG_WINDOW_TILE_DATA) {
                    tile_num as usize
                } else {
                    128 + ((tile_num as i8 as i16) + 128) as usize
                };

                let line = ((y % 8) * 2) as usize;
                let tile_mask = tile_num << 4;
                let data1 = self.vram[(tile_mask | line) & 0x1fff];
                let data2 = self.vram[(tile_mask | (line + 1)) & 0x1fff];

                let bit = (x % 8).wrapping_sub(7).wrapping_mul(0xff) as usize;
                // (self >> bit) & Self::one()
                /*let bit = (x % 8).wrapping_sub(7).wrapping_mul(0xff) as usize;
                let color_value = (data2.bit(bit) << 1) | data1.bit(bit);
                let raw_color = Color::from_u8(color_value);
                let color = self.bg_palette.get(&raw_color);
                bg_prio[i] = raw_color != Color::Off;
                pixels[i] = color; */
                //let color_l = if data2 & (0x80 >> bit) != 0 { 1 } else { 0 };
                //let color_h = if data1 & (0x80 >> bit) != 0 { 2 } else { 0 };
                let color_value =
                    ((data2 >> (7 - (x % 8))) & 1) << 1 | ((data1 >> (7 - (x % 8))) & 1);
                let raw_color = Color::from_u8(color_value);

                let color = self.bgp.get(&raw_color);
                bg_prio[i] = raw_color != Color::Off;
                pixels[i] = color;
                if color_value != 0 {
                    println!("{} {}", i, color_value)
                };
            }
        }
        if self.control.contains(LcdcFlags::OBJ_DISPLAY_ENABLE) {
            let size = if self.control.contains(LcdcFlags::OBJ_SIZE) {
                16
            } else {
                8
            };

            let current_line = self.ly;

            let mut sprites_to_draw: ArrayVec<[(usize, Sprite); 10]> = self
                .oam
                .chunks(4)
                .filter_map(|chunk| match chunk {
                    &[y, x, tile_num, flags] => {
                        let y = y.wrapping_sub(16);
                        let x = x.wrapping_sub(8);
                        let flags = SpriteFlags::from_bits_truncate(flags);
                        println!("sprite build");
                        if current_line.wrapping_sub(y) < size {
                            Some(Sprite {
                                y,
                                x,
                                tile: tile_num,
                                attr: flags,
                            })
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .take(10)
                .enumerate()
                .collect();

            sprites_to_draw.sort_by(|&(a_index, a), &(b_index, b)| {
                match a.x.cmp(&b.x) {
                    // If X coordinates are the same, use OAM index as priority (low index => draw last)
                    Ordering::Equal => a_index.cmp(&b_index).reverse(),
                    // Use X coordinate as priority (low X => draw last)
                    other => other.reverse(),
                }
            });

            for (_, sprite) in sprites_to_draw {
                let palette = if sprite.attr.contains(SpriteFlags::PALETTE) {
                    &self.obp1
                } else {
                    &self.obp0
                };
                let mut tile_num = sprite.tile as usize;
                let mut line = if sprite.attr.contains(SpriteFlags::FLIPY) {
                    size - current_line.wrapping_sub(sprite.y) - 1
                } else {
                    current_line.wrapping_sub(sprite.y)
                };
                if line >= 8 {
                    tile_num += 1;
                    line -= 8;
                }
                line *= 2;
                let tile_mask = tile_num << 4;
                let data1 = self.vram[(tile_mask | line as usize) & 0x1fff];
                let data2 = self.vram[(tile_mask | (line + 1) as usize) & 0x1fff];

                for x in (0..8).rev() {
                    let bit = if sprite.attr.contains(SpriteFlags::FLIPX) {
                        7 - x
                    } else {
                        x
                    } as usize;

                    //let color_l = if data2 & (0x80 >> bit) != 0 { 1 } else { 0 };
                    //let color_h = if data1 & (0x80 >> bit) != 0 { 2 } else { 0 };
                    //let color_value = color_h | color_l;
                    let color_value =
                        ((data2 >> (7 - (x % 8))) & 1) << 1 | ((data1 >> (7 - (x % 8))) & 1);
                    let raw_color = Color::from_u8(color_value);
                    let color = palette.get(&raw_color);
                    let target_x = sprite.x.wrapping_add(7 - x);
                    if target_x < SCREEN_WIDTH as u8
                        && raw_color != Color::Off
                        && (!sprite.attr.contains(SpriteFlags::PRIORITY)
                            || !bg_prio[target_x as usize])
                    {
                        pixels[target_x as usize] = color;
                        if color_value != 0 {
                            println!("{} {}", target_x, color_value)
                        };
                    }
                }
            }
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
            0xFF47 => self.bgp.bits,
            0xFF48 => self.obp0.bits,
            0xFF49 => self.obp1.bits,
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
            0xFF47 => self.bgp.set_bits(val),
            0xFF48 => self.obp0.set_bits(val),
            0xFF49 => self.obp1.set_bits(val),
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
                self.draw_line();
                self.update_mode(Mode::HBlank);
            }
            Mode::HBlank => {
                self.ly += 1;
                if self.ly < 144 {
                    self.update_mode(Mode::AccessOAM);
                } else {
                    self.updated = true;
                    self.update_mode(Mode::VBlank);
                }
                self.check_lyc();
            }
            Mode::VBlank => {
                self.ly += 1;
                if self.ly > 153 {
                    self.ly = 0;
                    //self.check_lyc();
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
