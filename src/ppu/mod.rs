use crate::mmu::memory::Memory;

// TODO: Look at doing Pixel FIFO - Rendering one line at a time is fine in most cases for now.
// Only a few games actually require pixel FIFO.

/// The Gameboy outputs a 160x144 pixel LCD screen.
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;
pub const SCREEN_PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

/// The Gameboy has three layers for rendering. Background, Window, and Sprites.
pub const BG_WIDTH: usize = 256;
pub const BG_HEIGHT: usize = 256;
pub const BG_PIXELS: usize = BG_WIDTH * BG_HEIGHT;
pub const BG_TILES: usize = 32 * 32;
pub const BG_MAP: usize = 32 * 32;
pub const WIN_WIDTH: usize = 256;
pub const WIN_HEIGHT: usize = 256;
pub const WIN_PIXELS: usize = WIN_WIDTH * WIN_HEIGHT;
pub const WIN_TILES: usize = 32 * 32;
pub const WIN_MAP: usize = 32 * 32;

/// The PPU had varying cycles depending on the mode it was in.
const ACCESS_OAM_CYCLES: u32 = 21;
const ACCESS_VRAM_CYCLES: u32 = 43;
const HBLANK_CYCLES: u32 = 50;
const VBLANK_CYCLES: u32 = 114;

/// PPU also handles VRAM and OAM memory.
pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;
pub const VRAM_SIZE: usize = 0x2000;
pub const OAM_SIZE: usize = 0xA0;
pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;

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

    /// The attributes of the sprite (Sprite Flags)
    /// Bit 7   OBJ-to-BG Priority (0=OBJ Above BG, 1=OBJ Behind BG color 1-3)
    /// Bit 6   Y flip          (0=Normal, 1=Vertically mirrored)
    /// Bit 5   X flip          (0=Normal, 1=Horizontally mirrored)
    /// Bit 4   Palette number  **DMG Only* (0=OBP0, 1=OBP1)
    /// Bit 0-3 CGB Only
    attr: u8,
    priority: bool,
    y_flip: bool,
    x_flip: bool,
    palette: bool,

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
        let priority = data[3] & 0x80 == 0x80;
        let y_flip = data[3] & 0x40 == 0x40;
        let x_flip = data[3] & 0x20 == 0x20;
        let palette = data[3] & 0x10 == 0x10;
        Self {
            x: data[0],
            y: data[1],
            tile_id: data[2],
            attr: data[3],
            priority,
            y_flip,
            x_flip,
            palette,
            size,
            tile,
        }
    }
}

/// During a scanline, the PPU enters multiple different modes.
/// There are 4 modes, each with a specific function.
enum PpuMode {
    /// Mode 0 - H-Blank
    /// This mode takes up the remainder of the scanline after the Drawing Mode finishes.
    /// This is more or less “padding” the duration of the scanline to a total of 456 T-Cycles.
    /// The PPU effectively pauses during this mode.
    HBlank,

    /// Mode 1 - V-Blank
    /// This mode is similar to H-Blank, in that it the PPU does not render to the LCD during its duration.
    /// However, instead of taking place at the end of every scanline, it is a much longer period at the
    /// end of every frame.
    ///
    /// As the Gameboy has a vertical resolution of 144 pixels, it would be expected that the amount of
    /// scanlines the PPU handles would be equal - 144 scanlines. However, this is not the case.
    /// In reality there are 154 scanlines, the 10 last of which being “pseudo-scanlines” during which
    /// no pixels are drawn as the PPU is in the V-Blank state during their duration.
    /// A V-Blank scanline takes the same amount of time as any other scanline - 456 T-Cycles.
    VBlank,

    /// Mode 2 - OAM Scan
    /// This mode is entered at the start of every scanline (except for V-Blank) before pixels are actually drawn to the screen.
    /// During this mode the PPU searches OAM memory for sprites that should be rendered on the current scanline and stores them in a buffer.
    /// This procedure takes a total amount of 80 T-Cycles, meaning that the PPU checks a new OAM entry every 2 T-Cycles.
    ///
    /// A sprite is only added to the buffer if all of the following conditions apply:
    ///     * Sprite X-Position must be greater than 0
    ///     * LY + 16 must be greater than or equal to Sprite Y-Position
    ///     * LY + 16 must be less than Sprite Y-Position + Sprite Height (8 in Normal Mode, 16 in Tall-Sprite-Mode)
    ///     * The amount of sprites already stored in the OAM Buffer must be less than 10
    OamScan,

    /// Mode 3 - Drawing
    /// The Drawing Mode is where the PPU transfers pixels to the LCD.
    /// The duration of this mode changes depending on multiple variables,
    /// such as background scrolling, the amount of sprites on the scanline, whether or not the window should be rendered, etc.
    Drawing,
}

/// LCD Control Register (LCDC - $FF40)
/// Bit 7  LCD Display Enable
///     Setting this bit to 0 disables the PPU entirely. The screen is turned off.
///
/// Bit 6  Window Tile Map Select
///     If set to 1, the Window will use the background map located at $9C00-$9FFF. Otherwise, it uses $9800-$9BFF.
///
/// Bit 5  Window Display Enable
///     Setting this bit to 0 hides the window layer entirely.
///
/// Bit 4  Tile Data Select
///     If set to 1, fetching Tile Data uses the 8000 method. Otherwise, the 8800 method is used.
///
/// Bit 3  BG Tile Map Select
///     If set to 1, the Background will use the background map located at $9C00-$9FFF. Otherwise, it uses $9800-$9BFF.
///
/// Bit 2  Sprite Size
///     If set to 1, sprites are displayed as 1x2 Tile (8x16 pixel) object. Otherwise, they're 1x1 Tile.
///
/// Bit 1  Sprite Enable
///     Sprites are only drawn to screen if this bit is set to 1.
///
/// Bit 0  BG/Window Enable
///     If this bit is set to 0, neither Background nor Window tiles are drawn. Sprites are unaffected
struct Lcdc {
    data: u8,
}

impl Lcdc {
    fn new() -> Self {
        Self { data: 0 }
    }

    fn set(&mut self, data: u8) {
        self.data = data;
    }

    /// LCDC.7 - LCD Display Enable
    /// This bit controls whether or not the PPU is active at all.
    /// The PPU only operates while this bit is set to 1.
    /// As soon as it is set to 0 the screen goes blank and the PPU stops all operation.
    /// The PPU also undergoes a “reset”.
    fn lcd_display_enable(&self) -> bool {
        self.data & (1 << 7) != 0
    }

    /// LCDC.6 - Window Tile Map Select
    /// This bit controls which Background Map is used to determine the tile numbers of the tiles displayed in the Window layer.
    /// If it is set to 1, the background map located at $9C00-$9FFF is used, otherwise it uses the one at $9800-$9BFF.
    fn window_tile_map_select(&self) -> bool {
        self.data * (1 << 6) != 0
    }

    /// LCDC.5 - Window Display Enable
    /// This bit controls whether or not the Window layer is rendered at all.
    /// If it is set to 0, everything Window-related can be ignored, as it is not rendered.
    /// Otherwise the Window renders as normal.
    fn window_display_enable(&self) -> bool {
        self.data & (1 << 5) != 0
    }

    /// LCDC.4 - Tile Data Select
    /// This bit determines which addressing mode to use for fetching Tile Data.
    /// If it is set to 1, the 8000 method is used. Otherwise, the 8800 method is used.
    fn tile_data_select(&self) -> bool {
        self.data * (1 << 4) != 0
    }

    /// LCDC.3 - BG Tile Map Select
    /// This bit controls which Background Map is used to determine the tile numbers of the tiles displayed in the Background layer.
    /// If it is set to 1, the background map located at $9C00-$9FFF is used, otherwise it uses the one at $9800-$9BFF.
    fn bg_tile_map_select(&self) -> bool {
        self.data & (1 << 3) != 0
    }

    /// LCDC.2 - Sprite Size
    /// As mentioned in the description of sprites above, there is a certain option which can enable “Tall Sprite Mode”.
    /// Setting this bit to 1 does so. In this mode, each sprite consists of two tiles on top of each other rather than one.
    fn sprite_size(&self) -> bool {
        self.data & (1 << 2) != 0
    }

    /// LCDC.1 - Sprite Enable
    /// This bit controls whether or not sprites are rendered at all.
    /// Setting this bit to 0 hides all sprites, otherwise they are rendered as normal.
    fn sprite_enable(&self) -> bool {
        self.data & (1 << 1) != 0
    }

    /// LCDC.0 - BG/Window Enable
    /// This bit controls whether or not Background and Window tiles are drawn.
    /// If it is set to 0, no Background or Window tiles are drawn and all pixels are drawn as white (Color 0).
    /// The only exception to this are sprites, as they are unaffected.
    fn bg_window_enable(&self) -> bool {
        self.data & (1 << 0) != 0
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

    /// Background Maps
    /// These keep track of the order tiles should be rendered in for the background and window layers.
    /// The VRAM sections $9800-$9BFF and $9C00-$9FFF each contain one of these background maps.
    /// The background map is made up of 32x32 bytes, representing tile numbers, organized row by row.
    background_map: Vec<u8>,
    window_map: Vec<u8>,

    /// The current PPU Mode
    mode: PpuMode,

    /// LCD Control Register (LCDC)
    lcdc: Lcdc,

    /// The PPU handles VRAM and OAM memory.
    /// VRAM is used to store the background and window tiles.
    /// OAM is used to store the sprite data.
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],

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
            background_map: vec![0; BG_MAP],
            window_map: vec![0; WIN_MAP],
            mode: PpuMode::OamScan,
            lcdc: Lcdc::new(),
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
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
