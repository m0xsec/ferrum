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
const DARK_GRAY: u32 = 0x00555555u32;
const LIGHT_GRAY: u32 = 0x00AAAAAAu32;
const WHITE: u32 = 0x00FFFFFFu32;

/// Gameboy DMG-01 colors
/// https://gbdev.io/pandocs/Palettes.html
/// Value   Color
/// 0       White
/// 1       Light Gray
/// 2       Dark Gray
/// 3       Black
enum Color {
    White,
    LightGray,
    DarkGray,
    Black,
}

impl Color {
    /// Convert a u8 to a Color
    fn from_u8(val: u8) -> Self {
        match val {
            0 => Color::White,
            1 => Color::LightGray,
            2 => Color::DarkGray,
            3 => Color::Black,
            _ => panic!("Invalid color value: {}", val),
        }
    }

    /// Convert a Color to a u32
    /// This is used to convert from Gameboy colors to u32 colors for rendering.
    fn to_u32(&self) -> u32 {
        match self {
            Color::White => WHITE,
            Color::LightGray => LIGHT_GRAY,
            Color::DarkGray => DARK_GRAY,
            Color::Black => BLACK,
        }
    }
}

/// Tiles are 8x8 pixels.
/// 2 bits are needed to store color data for a single pixel.
/// 2 bytes make up a row of 8 pixels.
/// Each bit of the first byte is combined with the bit at
/// the same position of the second byte to calculate a color number:
///
/// 0xA5:    1  0  1  0  0  1  0  1
/// 0xC3:    1  1  0  0  0  0  1  1
///
/// Encoded: 11 10 01 00 00 01 10 11
///
/// In memory, Tiles are stored as 16 bytes using the encoded method above.
/// The first 2 bytes represent the first row of 8 pixels, the next 2 the second row, etc.
/// https://pixelbits.16-b.it/GBEDG/ppu/#The-Concept-of-Tiles
struct Tile {
    data: [u8; 16],
}

impl Tile {
    /// Create a new Tile from a slice of 16 bytes.
    fn new(data: &[u8]) -> Self {
        let mut tile = Tile { data: [0; 16] };
        tile.data.copy_from_slice(data);
        tile
    }
    /// Get the color of a pixel at a given x,y coordinate.
    fn get_pixel(&self, x: usize, y: usize) -> Color {
        let byte1 = self.data[y * 2];
        let byte2 = self.data[y * 2 + 1];

        let bit1 = (byte1 >> (7 - x)) & 0x01;
        let bit2 = (byte2 >> (7 - x)) & 0x01;

        Color::from_u8(bit1 | (bit2 << 1))
    }
}

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
