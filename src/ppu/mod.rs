use std::default;

use crate::mmu::memory::Memory;

/// The Gameboy outputs a 160x144 pixel LCD screen.
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;
pub const SCREEN_PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

/// The Gameboy has three layers for rendering. Background, Window, and Sprites.
pub const BG_WIDTH: usize = 256;
pub const BG_HEIGHT: usize = 256;
pub const BG_PIXELS: usize = BG_WIDTH * BG_HEIGHT;
pub const BG_TILES: usize = 32 * 32;
pub const WIN_WIDTH: usize = 256;
pub const WIN_HEIGHT: usize = 256;
pub const WIN_PIXELS: usize = WIN_WIDTH * WIN_HEIGHT;
pub const WIN_TILES: usize = 32 * 32;

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
    /// This is used to c onvert from Gameboy colors to u32 colors for rendering.
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
#[derive(Clone, Copy)]
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

/// Each sprite can be 8x8 or 8x16 pixels (1x1 tile or 1x2 tiles) depending on the sprite size flag (LCDC.2).
/// NOTE: This is universal for all sprites in the loaded ROM.
#[derive(Default, Clone, Copy)]
enum SpriteSize {
    /// 8x8 pixels (1x1 tile)
    #[default]
    Small,

    /// 8x16 pixels (1x2 tiles)
    Large,
}

/// The sprite layer is made up of 40 sprites that are stored in OAM.
/// Each sprite can be 8x8 or 8x16 pixels (1x1 tile or 1x2 tiles) depending on the sprite size flag (LCDC.2).
/// Sprites are rendered on top of the background and window layers.
/// Sprites can be rendered behind the background and window layers by setting the priority flag (OAM.7).
/// Sprites can be flipped horizontally and vertically.
/// Sprites can be colored using one of the four palettes.
/// Sprites can be hidden by setting the hidden flag (OAM.1).
/// Sprites can be moved off screen by setting the x position to 0 or 160, or the y position to 0 or 144.
#[derive(Clone)]
struct Sprite {
    /// The x position of the sprite.
    x: u8,

    /// The y position of the sprite.
    y: u8,

    /// The tile number of the sprite.
    tile_id: u8,

    /// The attributes of the sprite.
    attr: u8,

    /// The sprite size (determined by the LCDC.2 flag).
    size: SpriteSize,

    /// The tile data of the sprite.
    tile: Vec<Tile>,
}

impl Sprite {
    /// Create a new Sprite from a slice of 4 bytes.
    /// Also using the sprite size flag (LCDC.2) to determine the sprite size.
    fn new(data: &[u8], size: SpriteSize) -> Self {
        let mut tile = Vec::new();
        match size {
            SpriteSize::Small => {
                tile.push(Tile::new(&[0; 16]));
            }
            SpriteSize::Large => {
                tile.push(Tile::new(&[0; 16]));
                tile.push(Tile::new(&[0; 16]));
            }
        }
        Self {
            x: data[0],
            y: data[1],
            tile_id: data[2],
            attr: data[3],
            size,
            tile,
        }
    }
}

/// PPU (Picture Processing Unit)
pub struct Ppu {
    /// The PPU has 3 layers, Background, Window, and Sprites.
    /// Each layer can be enabled or disabled.
    bg_enabled: bool,
    window_enabled: bool,
    sprite_enabled: bool,

    /// The background layer is made up of 32x32 tiles (256x256 pixels).
    /// The Gameboy can only display 20x18 tiles (160x144 pixels) at a time (this is the viewport).
    /// The offsets of the viewport are determined by the scroll registers (SCX, SCY).
    bg_tiles: Vec<Tile>,

    /// The window layer is made up of 32x32 tiles (256x256 pixels).
    /// The Gameboy can only display 20x18 tiles (160x144 pixels) at a time (this is the viewport).
    /// The offsets of the viewport are determined by the window position registers (WX, WY).
    /// The window layer is rendered on top of the background layer, think of it like an overlay.
    window_tiles: Vec<Tile>,

    /// The sprite layer is made up of 40 sprites that are stored in OAM.
    /// Each sprite can be 8x8 or 8x16 pixels (1x1 or 1x2 Tiles) depending on the sprite size flag (LCDC.2).
    sprites: Vec<Sprite>,

    /// Rendering buffer of the viewport.
    /// u32 vector of size 160x144. Each u32 represents the color of a pixel.
    pub viewport_buffer: Vec<u32>,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            bg_enabled: false,
            window_enabled: false,
            sprite_enabled: false,
            bg_tiles: vec![Tile::new(&[0; 16]); BG_TILES],
            window_tiles: vec![Tile::new(&[0; 16]); WIN_TILES],
            //sprites: vec![Sprite::new(&[0; 4], SpriteSize::Small); 40],
            sprites: vec![],
            viewport_buffer: vec![BLACK; SCREEN_PIXELS],
        }
    }

    /// Initialize sprites vector once we know the sprite size.
    fn init_sprites(&mut self, size: SpriteSize) {
        self.sprites = vec![Sprite::new(&[0; 4], size); 40];
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
