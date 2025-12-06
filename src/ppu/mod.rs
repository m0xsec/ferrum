use std::{cell::RefCell, rc::Rc};

use crate::cpu::interrupts::{Flags, InterruptFlags};
use crate::mmu::memory::Memory;

/// The Game Boy outputs a 160x144 pixel LCD screen.
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;
pub const SCREEN_PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;
pub const VRAM_SIZE: usize = 0x2000;

pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;
pub const OAM_SIZE: usize = 0xA0;

const SCANLINE_CYCLES: u32 = 456;
const OAM_CYCLES: u32 = 80;
const TRANSFER_CYCLES: u32 = 172;
const UNDEFINED_READ: u8 = 0xFF;

const BLACK: u32 = 0x00000000;
const DARK_GRAY: u32 = 0x00555555;
const LIGHT_GRAY: u32 = 0x00AAAAAA;
const WHITE: u32 = 0x00FFFFFF;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PpuMode {
    HBlank,
    VBlank,
    OamScan,
    Drawing,
}

#[derive(Clone, Copy)]
enum Color {
    White,
    LightGray,
    DarkGray,
    Black,
}

impl Color {
    fn from_u8(val: u8) -> Self {
        match val {
            0 => Color::White,
            1 => Color::LightGray,
            2 => Color::DarkGray,
            3 => Color::Black,
            _ => panic!("Invalid color value: {}", val),
        }
    }

    fn to_u32(self) -> u32 {
        match self {
            Color::White => WHITE,
            Color::LightGray => LIGHT_GRAY,
            Color::DarkGray => DARK_GRAY,
            Color::Black => BLACK,
        }
    }
}

#[derive(Default)]
struct Lcdc {
    data: u8,
}

impl Lcdc {
    fn set(&mut self, data: u8) {
        self.data = data;
    }

    fn lcd_display_enable(&self) -> bool {
        self.data & (1 << 7) != 0
    }

    fn window_tile_map_select(&self) -> bool {
        self.data & (1 << 6) != 0
    }

    fn window_display_enable(&self) -> bool {
        self.data & (1 << 5) != 0
    }

    fn tile_data_select(&self) -> bool {
        self.data & (1 << 4) != 0
    }

    fn bg_tile_map_select(&self) -> bool {
        self.data & (1 << 3) != 0
    }

    fn sprite_size(&self) -> bool {
        self.data & (1 << 2) != 0
    }

    fn sprite_enable(&self) -> bool {
        self.data & (1 << 1) != 0
    }

    fn bg_window_enable(&self) -> bool {
        self.data & 0x01 != 0
    }
}

#[derive(Default)]
struct Stat {
    data: u8,
}

impl Stat {
    fn set(&mut self, data: u8) {
        // Only the interrupt enable bits (6-3) are writable by the CPU.
        self.data = (self.data & 0x07) | (data & 0xF8);
        self.data |= 0x80; // bit7 always 1
    }

    fn update(&mut self, mode: PpuMode, ly: u8, lyc: u8) {
        let mut value = self.data & 0xF8;
        value |= 0x80;

        if ly == lyc {
            value |= 1 << 2;
        }

        value |= match mode {
            PpuMode::HBlank => 0,
            PpuMode::VBlank => 1,
            PpuMode::OamScan => 2,
            PpuMode::Drawing => 3,
        };

        self.data = value;
    }

    fn lyc_ly_stat_interrupt_enable(&self) -> bool {
        self.data & (1 << 6) != 0
    }

    fn mode_2_stat_interrupt_enable(&self) -> bool {
        self.data & (1 << 5) != 0
    }

    fn mode_1_stat_interrupt_enable(&self) -> bool {
        self.data & (1 << 4) != 0
    }

    fn mode_0_stat_interrupt_enable(&self) -> bool {
        self.data & (1 << 3) != 0
    }
}

#[derive(Clone, Copy)]
struct SpriteAttrs {
    priority: bool,
    y_flip: bool,
    x_flip: bool,
    palette1: bool,
}

pub struct Ppu {
    vram: Rc<RefCell<[u8; VRAM_SIZE]>>,
    oam: Rc<RefCell<[u8; OAM_SIZE]>>,
    if_: Rc<RefCell<InterruptFlags>>,

    lcdc: Lcdc,
    stat: Stat,

    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    wy: u8,
    wx: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,

    mode: PpuMode,
    dots: u32,
    window_line: u8,
    lcd_enabled: bool,

    pub viewport_buffer: Vec<Vec<u32>>,
    pub updated: bool,
}

impl Ppu {
    pub fn new(if_: Rc<RefCell<InterruptFlags>>) -> Self {
        Self {
            vram: Rc::new(RefCell::new([0; VRAM_SIZE])),
            oam: Rc::new(RefCell::new([0; OAM_SIZE])),
            if_,
            lcdc: Lcdc::default(),
            stat: Stat { data: 0x80 },
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            wy: 0,
            wx: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            mode: PpuMode::OamScan,
            dots: 0,
            window_line: 0,
            lcd_enabled: false,
            viewport_buffer: vec![vec![WHITE; SCREEN_WIDTH]; SCREEN_HEIGHT],
            updated: false,
        }
    }

    fn request_stat_if(&mut self, condition: bool) {
        if condition {
            self.if_.borrow_mut().set(Flags::LCDStat);
        }
    }

    fn handle_lyc(&mut self) {
        if self.ly == self.lyc && self.stat.lyc_ly_stat_interrupt_enable() {
            self.if_.borrow_mut().set(Flags::LCDStat);
        }
    }

    fn disable_lcd(&mut self) {
        self.lcd_enabled = false;
        self.mode = PpuMode::HBlank;
        self.dots = 0;
        self.ly = 0;
        self.window_line = 0;
        self.stat.update(self.mode, self.ly, self.lyc);
    }

    fn enable_lcd(&mut self) {
        self.lcd_enabled = true;
        self.mode = PpuMode::OamScan;
        self.dots = 0;
        self.ly = 0;
        self.window_line = 0;
        self.stat.update(self.mode, self.ly, self.lyc);
        self.request_stat_if(self.stat.mode_2_stat_interrupt_enable());
        self.handle_lyc();
    }

    fn fetch_tile_pixel(
        &self,
        tile_id: u8,
        tile_line: u8,
        x_in_tile: u8,
        unsigned_mode: bool,
    ) -> u8 {
        let base = if unsigned_mode {
            0x8000u16 + tile_id as u16 * 16
        } else {
            let id = tile_id as i8 as i16;
            (0x9000i32 + (id as i32) * 16) as u16
        };

        let addr = base + tile_line as u16 * 2;
        let vram = self.vram.borrow();
        let lo = vram[addr as usize - VRAM_START as usize];
        let hi = vram[addr as usize + 1 - VRAM_START as usize];

        let bit = 7 - x_in_tile;
        ((lo >> bit) & 1) | (((hi >> bit) & 1) << 1)
    }

    fn map_palette(&self, palette: u8, color_id: u8) -> Color {
        let shade = (palette >> (color_id * 2)) & 0x03;
        Color::from_u8(shade)
    }

    fn collect_line_sprites(&self) -> Vec<(u8, u8, u8, SpriteAttrs)> {
        if !self.lcdc.sprite_enable() {
            return Vec::new();
        }

        let sprite_height = if self.lcdc.sprite_size() { 16 } else { 8 };
        let mut sprites = Vec::new();

        for i in 0..40 {
            let base = i * 4;
            let data = self.oam.borrow();
            let y = data[base] as i16 - 16;
            let x = data[base + 1] as i16 - 8;
            let tile = data[base + 2];
            let attr = data[base + 3];
            drop(data);

            let line = self.ly as i16;
            if line < y || line >= y + sprite_height {
                continue;
            }

            sprites.push((
                x as u8,
                y as u8,
                tile,
                SpriteAttrs {
                    priority: attr & 0x80 != 0,
                    y_flip: attr & 0x40 != 0,
                    x_flip: attr & 0x20 != 0,
                    palette1: attr & 0x10 != 0,
                },
            ));

            if sprites.len() == 10 {
                break;
            }
        }

        sprites
    }

    fn render_scanline(&mut self) {
        if (self.ly as usize) < SCREEN_HEIGHT {
            let window_active =
                self.lcdc.window_display_enable() && self.ly >= self.wy && self.wx <= 166;
            let window_x_origin = self.wx.wrapping_sub(7);

            let mut used_window = false;
            let line_sprites = self.collect_line_sprites();

            for x in 0..SCREEN_WIDTH {
                let mut bg_color_id = 0;
                if self.lcdc.bg_window_enable() {
                    let using_window = window_active && (x as u8) >= window_x_origin;
                    let (tile_x, tile_y, tile_map_base, unsigned_mode) = if using_window {
                        used_window = true;
                        let wx = x as i16 - window_x_origin as i16;
                        let wy = self.window_line as i16;
                        (
                            wx as u8,
                            wy as u8,
                            if self.lcdc.window_tile_map_select() {
                                0x9C00
                            } else {
                                0x9800
                            },
                            self.lcdc.tile_data_select(),
                        )
                    } else {
                        (
                            self.scx.wrapping_add(x as u8),
                            self.scy.wrapping_add(self.ly),
                            if self.lcdc.bg_tile_map_select() {
                                0x9C00
                            } else {
                                0x9800
                            },
                            self.lcdc.tile_data_select(),
                        )
                    };

                    let tile_index = ((tile_y / 8) as u16) * 32 + (tile_x / 8) as u16;
                    let tile_id =
                        self.vram.borrow()[(tile_map_base + tile_index - VRAM_START) as usize];
                    let tile_line = tile_y % 8;
                    let x_in_tile = tile_x % 8;
                    bg_color_id =
                        self.fetch_tile_pixel(tile_id, tile_line, x_in_tile, unsigned_mode);
                }

                let mut final_color = self.map_palette(self.bgp, bg_color_id);

                if self.lcdc.sprite_enable() {
                    for (sprite_x, sprite_y, tile_id, attrs) in &line_sprites {
                        let sprite_height = if self.lcdc.sprite_size() { 16 } else { 8 };
                        if *sprite_x == 0
                            || (x as i16) < (*sprite_x as i16)
                            || (x as i16) >= (*sprite_x as i16 + 8)
                        {
                            continue;
                        }

                        let mut line = self.ly.wrapping_sub(*sprite_y);
                        if attrs.y_flip {
                            line = sprite_height as u8 - 1 - line;
                        }

                        let mut pixel_x = (x as u8).wrapping_sub(*sprite_x);
                        if attrs.x_flip {
                            pixel_x = 7 - pixel_x;
                        }

                        let mut index = *tile_id;
                        if sprite_height == 16 {
                            index &= 0xFE;
                            if line >= 8 {
                                index = index.wrapping_add(1);
                            }
                            line %= 8;
                        }

                        let color_id = self.fetch_tile_pixel(index, line, pixel_x, true);
                        if color_id == 0 {
                            continue;
                        }

                        if attrs.priority && self.lcdc.bg_window_enable() && bg_color_id != 0 {
                            break;
                        }

                        let palette = if attrs.palette1 { self.obp1 } else { self.obp0 };
                        final_color = self.map_palette(palette, color_id);
                        break;
                    }
                }

                self.viewport_buffer[self.ly as usize][x] = final_color.to_u32();
            }

            if window_active && used_window {
                self.window_line = self.window_line.wrapping_add(1);
            }
        }
    }

    fn step(&mut self) {
        if !self.lcdc.lcd_display_enable() {
            self.disable_lcd();
            return;
        }

        if !self.lcd_enabled {
            self.enable_lcd();
            return;
        }

        self.dots += 1;

        match self.mode {
            PpuMode::OamScan => {
                if self.dots == OAM_CYCLES {
                    self.mode = PpuMode::Drawing;
                    self.stat.update(self.mode, self.ly, self.lyc);
                }
            }
            PpuMode::Drawing => {
                if self.dots == OAM_CYCLES + TRANSFER_CYCLES {
                    self.render_scanline();
                    self.mode = PpuMode::HBlank;
                    self.stat.update(self.mode, self.ly, self.lyc);
                    self.request_stat_if(self.stat.mode_0_stat_interrupt_enable());
                }
            }
            PpuMode::HBlank => {
                if self.dots == SCANLINE_CYCLES {
                    self.dots = 0;
                    self.ly = self.ly.wrapping_add(1);
                    self.handle_lyc();

                    if self.ly == 144 {
                        self.mode = PpuMode::VBlank;
                        self.updated = true;
                        self.if_.borrow_mut().set(Flags::VBlank);
                        self.request_stat_if(self.stat.mode_1_stat_interrupt_enable());
                    } else {
                        self.mode = PpuMode::OamScan;
                        self.request_stat_if(self.stat.mode_2_stat_interrupt_enable());
                    }

                    self.stat.update(self.mode, self.ly, self.lyc);
                }
            }
            PpuMode::VBlank => {
                if self.dots == SCANLINE_CYCLES {
                    self.dots = 0;
                    self.ly = self.ly.wrapping_add(1);
                    self.handle_lyc();

                    if self.ly > 153 {
                        self.ly = 0;
                        self.window_line = 0;
                        self.mode = PpuMode::OamScan;
                        self.stat.update(self.mode, self.ly, self.lyc);
                        self.request_stat_if(self.stat.mode_2_stat_interrupt_enable());
                    } else {
                        self.stat.update(self.mode, self.ly, self.lyc);
                    }
                }
            }
        }
    }
}

impl Memory for Ppu {
    fn read8(&self, addr: u16) -> u8 {
        match addr {
            VRAM_START..=VRAM_END => {
                if self.mode == PpuMode::Drawing {
                    return UNDEFINED_READ;
                }
                self.vram.borrow()[(addr - VRAM_START) as usize]
            }
            OAM_START..=OAM_END => {
                if matches!(self.mode, PpuMode::Drawing | PpuMode::OamScan) {
                    return UNDEFINED_READ;
                }
                self.oam.borrow()[(addr - OAM_START) as usize]
            }
            0xFF40 => self.lcdc.data,
            0xFF41 => self.stat.data,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF46 => UNDEFINED_READ,
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
            VRAM_START..=VRAM_END => {
                if self.mode != PpuMode::Drawing {
                    self.vram.borrow_mut()[(addr - VRAM_START) as usize] = val;
                }
            }
            OAM_START..=OAM_END => {
                if !matches!(self.mode, PpuMode::Drawing | PpuMode::OamScan) {
                    self.oam.borrow_mut()[(addr - OAM_START) as usize] = val;
                }
            }
            0xFF40 => {
                let was_enabled = self.lcdc.lcd_display_enable();
                self.lcdc.set(val);
                if was_enabled && !self.lcdc.lcd_display_enable() {
                    self.disable_lcd();
                } else if !was_enabled && self.lcdc.lcd_display_enable() {
                    self.enable_lcd();
                }
            }
            0xFF41 => {
                self.stat.set(val);
            }
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => {
                // Read only
            }
            0xFF45 => {
                self.lyc = val;
                self.handle_lyc();
            }
            0xFF47 => self.bgp = val,
            0xFF48 => self.obp0 = val,
            0xFF49 => self.obp1 = val,
            0xFF4A => self.wy = val,
            0xFF4B => self.wx = val,
            _ => {}
        }
    }

    fn read16(&self, addr: u16) -> u16 {
        u16::from(self.read8(addr)) | (u16::from(self.read8(addr.wrapping_add(1))) << 8)
    }

    fn write16(&mut self, addr: u16, val: u16) {
        self.write8(addr, (val & 0xFF) as u8);
        self.write8(addr.wrapping_add(1), (val >> 8) as u8);
    }

    fn cycle(&mut self, cycles: u32) -> u32 {
        for _ in 0..cycles {
            self.step();
        }
        self.stat.update(self.mode, self.ly, self.lyc);
        cycles
    }
}
