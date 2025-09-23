use std::{cell::RefCell, rc::Rc};

use log::warn;

use crate::{
    cpu::interrupts::{Flags, InterruptFlags},
    mmu::memory::Memory,
};

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;
pub const SCREEN_PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT;
const VRAM_SIZE: usize = 0x2000;
const OAM_SIZE: usize = 0xA0;

// PPU Timings (in dots/T-cycles)
const DOTS_PER_SCANLINE: u32 = 456;
const OAM_SEARCH_DURATION: u32 = 80;
const PIXEL_TRANSFER_DURATION: u32 = 172; 

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PpuMode {
    HBlank = 0,
    VBlank = 1,
    OamSearch = 2,
    PixelTransfer = 3,
}

pub struct Ppu {
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],

    // I/O Registers (0xFF40 - 0xFF4B)
    lcdc: u8, // 0xFF40
    stat: u8, // 0xFF41
    scy: u8,  // 0xFF42
    scx: u8,  // 0xFF43
    ly: u8,   // 0xFF44
    lyc: u8,  // 0xFF45
    bgp: u8,  // 0xFF47
    obp0: u8, // 0xFF48
    obp1: u8, // 0xFF49
    wy: u8,   // 0xFF4A
    wx: u8,   // 0xFF4B

    // PPU State
    mode: PpuMode,
    dots: u32,
    
    // --- Window State Tracking (Accurate implementation) ---
    // Internal counter for the window's vertical position.
    window_line_counter: u8,
    // Tracks if LY==WY condition was met this frame (latched).
    window_wy_triggered: bool,
    
    // Internal line for tracking STAT interrupt conditions (edge triggering)
    stat_irq_line: bool,
    
    pub framebuffer: Vec<u32>,
    pub updated: bool,
    

    // Reference to interrupts
    if_: Rc<RefCell<InterruptFlags>>,
}

impl Ppu {
    pub fn new(if_: Rc<RefCell<InterruptFlags>>) -> Self {
        Self {
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],

            // Initialize to DMG Power-On Defaults (LCD Disabled)
            lcdc: 0x00,
            stat: 0x80, // Bit 7 always set
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0x00,
            obp0: 0x00,
            obp1: 0x00,
            wy: 0,
            wx: 0,

            mode: PpuMode::HBlank,
            dots: 0,
            window_line_counter: 0,
            window_wy_triggered: false,
            stat_irq_line: false,
            framebuffer: vec![Self::get_rgb_color(0); SCREEN_WIDTH * SCREEN_HEIGHT], 
            if_,
            updated: false,
        }
    }

    fn is_lcd_enabled(&self) -> bool {
        (self.lcdc & 0x80) != 0
    }

    // --- PPU Tick Function (State Machine) ---

    pub fn cycle(&mut self, cpu_cycles: u32) {
        // The PPU runs at 4MHz, so we need to run it 4 times for each CPU cycle.
        for _ in 0..(cpu_cycles * 4) {
            self.tick();
        }
    }

    pub fn tick(&mut self) {
        if !self.is_lcd_enabled() {
            return;
        }

        self.dots += 1;

        match self.mode {
            PpuMode::OamSearch => {
                if self.dots >= OAM_SEARCH_DURATION {
                    self.set_mode(PpuMode::PixelTransfer);
                }
            }
            PpuMode::PixelTransfer => {
                if self.dots >= OAM_SEARCH_DURATION + PIXEL_TRANSFER_DURATION {
                    self.render_scanline();
                    self.set_mode(PpuMode::HBlank);
                }
            }
            PpuMode::HBlank => {
                if self.dots >= DOTS_PER_SCANLINE {
                    self.ly += 1;
                    self.dots -= DOTS_PER_SCANLINE;

                    if self.ly == 144 {
                        // Transition to Mode 1 (VBlank)
                        self.set_mode(PpuMode::VBlank);
                        self.updated = true; // Frame is ready
                        // Request VBlank Interrupt (Crucial for ROM timing/animation)
                        //self.request_vblank = true;
                        // Request VBlank interrupt
                        self.if_.borrow_mut().set(Flags::VBlank);
                    } else {
                        // Start next scanline (Mode 2)
                        self.set_mode(PpuMode::OamSearch);
                        self.check_wy_trigger();
                    }
                    self.check_lyc();
                }
            }
            PpuMode::VBlank => {
                if self.dots >= DOTS_PER_SCANLINE {
                    self.ly += 1;
                    self.dots -= DOTS_PER_SCANLINE;

                    if self.ly > 153 {
                        // End of VBlank, reset for the new frame
                        self.ly = 0;
                        //self.updated = false;
                        self.window_line_counter = 0;
                        self.window_wy_triggered = false;
                        self.set_mode(PpuMode::OamSearch);
                        // Check WY condition for LY=0
                        self.check_wy_trigger();
                    }
                    self.check_lyc();
                }
            }
        }
    }

    // FIX: Checks if the WY condition (LY == WY) is met.
    // This comparison happens independently of the Window Enable bit (LCDC.5).
    fn check_wy_trigger(&mut self) {
        if self.ly >= 144 {
            return;
        }

        // If the current scanline matches WY, the trigger is latched for the frame.
        if self.ly == self.wy {
            self.window_wy_triggered = true;
        }
    }

    // (set_mode, check_lyc, update_stat_irq, handle_lcd_disable/enable remain robust)

    fn set_mode(&mut self, mode: PpuMode) {
        self.mode = mode;
        self.stat = (self.stat & 0xFC) | (mode as u8);
        self.update_stat_irq();
    }

    fn check_lyc(&mut self) {
        if self.ly == self.lyc {
            self.stat |= 0x04;
        } else {
            self.stat &= !0x04;
        }
        self.update_stat_irq();
    }

    fn update_stat_irq(&mut self) {
        if !self.is_lcd_enabled() {
            self.stat_irq_line = false;
            return;
        }

        let lyc_check = (self.stat & 0x40) != 0 && (self.stat & 0x04) != 0;
        let mode2_check = (self.stat & 0x20) != 0 && self.mode == PpuMode::OamSearch;
        let mode1_check = (self.stat & 0x10) != 0 && self.mode == PpuMode::VBlank;
        let mode0_check = (self.stat & 0x08) != 0 && self.mode == PpuMode::HBlank;

        let irq = lyc_check || mode2_check || mode1_check || mode0_check;

        // STAT interrupt is triggered only on the rising edge
        if irq && !self.stat_irq_line {
            //self.request_stat = true;
            self.if_.borrow_mut().set(Flags::LCDStat);
        }
        self.stat_irq_line = irq;
    }

    // --- LCD Control ---

    fn handle_lcd_disable(&mut self) {
        self.ly = 0;
        self.dots = 0;
        self.window_line_counter = 0;
        self.window_wy_triggered = false;
        self.mode = PpuMode::HBlank;
        self.stat &= 0xF8; 
        self.stat_irq_line = false;
        for pixel in self.framebuffer.iter_mut() {
            *pixel = Self::get_rgb_color(0);
        }
    }

    fn handle_lcd_enable(&mut self) {
        self.dots = 0;
        self.ly = 0;
        self.window_line_counter = 0;
        self.window_wy_triggered = false;
        self.set_mode(PpuMode::OamSearch);
        self.check_lyc();
        self.check_wy_trigger();
    }

    // --- Rendering Functions ---

    fn render_scanline(&mut self) {
        let mut bg_win_indices = [0u8; SCREEN_WIDTH];

        // LCDC Bit 0: BG/Window Master Enable (DMG)
        if (self.lcdc & 0x01) != 0 {
            self.render_bg_and_window(&mut bg_win_indices);
        } else {
            self.fill_line_bg_color0();
        }
        
        // LCDC Bit 1: OBJ (Sprite) Display Enable
        if (self.lcdc & 0x02) != 0 {
            self.render_sprites(&bg_win_indices);
        }
    }

    fn fill_line_bg_color0(&mut self) {
        let color = self.get_palette_color(self.bgp, 0);
        let y = self.ly as usize;
        for x in 0..SCREEN_WIDTH {
             self.set_pixel(x, y, color);
        }
    }

    // Renders both background and window
    fn render_bg_and_window(&mut self, bg_win_indices: &mut [u8; SCREEN_WIDTH]) {
        let y = self.ly as usize;

        // Background Setup
        let bg_tile_map_offset = if (self.lcdc & 0x08) != 0 { 0x1C00 } else { 0x1800 };
        let scrolled_y = self.ly.wrapping_add(self.scy);
        
        // --- Window Setup (Accurate Logic) ---
        let win_tile_map_offset = if (self.lcdc & 0x40) != 0 { 0x1C00 } else { 0x1800 };
        
        // The window is active on this scanline if:
        // 1. The WY trigger was latched earlier this frame (window_wy_triggered).
        // 2. The window is currently enabled (LCDC.5).
        // 3. FIX: WX is within the valid range (WX < 167).
        let window_active_on_scanline = self.window_wy_triggered 
                                        && (self.lcdc & 0x20) != 0
                                        && self.wx < 167;

        // WX is offset by 7 (WX=7 is the left edge)
        let window_x_start = self.wx.saturating_sub(7);
        let current_window_y = self.window_line_counter;

        // Tile Data Select
        let use_8000_mode = (self.lcdc & 0x10) != 0;

        for x in 0..SCREEN_WIDTH {
            let (tile_map_offset, pixel_y, pixel_x) = 
                // Check if the window is active and the current pixel is within the window horizontally
                if window_active_on_scanline && x as u8 >= window_x_start {
                    // Inside the window (Independent of SCX/SCY)
                    (win_tile_map_offset, current_window_y, (x as u8 - window_x_start))
                } else {
                    // Inside the background
                    // Apply SCX for horizontal scrolling
                    let scrolled_x = (x as u8).wrapping_add(self.scx);
                    (bg_tile_map_offset, scrolled_y, scrolled_x)
                };

            // 1. Get Tile Index
            let tile_row = (pixel_y / 8) as u16;
            let tile_col = (pixel_x / 8) as u16;
            let map_vram_offset = tile_map_offset + tile_row * 32 + tile_col;
            let tile_index = self.internal_read_vram(map_vram_offset);

            // 2. Get Tile Data Address (VRAM Offset 0x0000-0x1FFF)
            let tile_data_vram_offset = if use_8000_mode {
                (tile_index as u16 * 16)
            } else {
                let signed_index = tile_index as i8;
                // 0x1000 + (signed_offset * 16). Use i32 for safe calculation.
                ((0x1000_i32 + (signed_index as i32 * 16)) as u16)
            };

            // 3. Fetch Pixel Data (2bpp)
            let line_offset = (pixel_y % 8) as u16 * 2;
            let data1 = self.internal_read_vram(tile_data_vram_offset + line_offset);
            let data2 = self.internal_read_vram(tile_data_vram_offset + line_offset + 1);

            // 4. Decode Color Index (data2=MSB, data1=LSB)
            let bit = 7 - (pixel_x % 8);
            let color_index = (((data2 >> bit) & 1) << 1) | ((data1 >> bit) & 1);

            bg_win_indices[x] = color_index;

            // 5. Apply Palette and Draw
            let color = self.get_palette_color(self.bgp, color_index);
            self.set_pixel(x, y, color);
        }

        // If the window was active on this scanline (all conditions met, including WX < 167), 
        // increment the counter for the next line.
        if window_active_on_scanline {
            self.window_line_counter = self.window_line_counter.wrapping_add(1);
        }
    }

    // Sprite (OBJ) rendering (Logic remains robust, included for completeness)
    fn render_sprites(&mut self, bg_win_indices: &[u8; SCREEN_WIDTH]) {
        // LCDC Bit 2: OBJ Size (0=8x8, 1=8x16)
        let sprite_height = if (self.lcdc & 0x04) != 0 { 16 } else { 8 };
        let y = self.ly as usize;

        // 1. OAM Scan: Find up to 10 visible sprites.
        let mut visible_sprites = Vec::with_capacity(10);
        for i in 0..40 {
            let oam_offset = (i * 4) as u16;
            let sprite_y = self.internal_read_oam(oam_offset).wrapping_sub(16);
            
            if self.ly >= sprite_y && self.ly < sprite_y.wrapping_add(sprite_height) {
                let sprite_x = self.internal_read_oam(oam_offset + 1).wrapping_sub(8);
                let tile_index = self.internal_read_oam(oam_offset + 2);
                let attributes = self.internal_read_oam(oam_offset + 3);
                
                visible_sprites.push((sprite_x, sprite_y, tile_index, attributes, i));
                
                if visible_sprites.len() == 10 {
                    break;
                }
            }
        }

        // 2. Priority Sorting (DMG specific: X coordinate priority, then OAM index)
        // Sort descending (back-to-front drawing)
        visible_sprites.sort_by(|a, b| {
            match b.0.cmp(&a.0) { // Compare X coordinates
                std::cmp::Ordering::Equal => b.4.cmp(&a.4), // Compare OAM indices
                other => other,
            }
        });

        // 3. Render sprites (back-to-front)
        for (sprite_x, sprite_y, tile_index, attributes, _) in visible_sprites {
            // Sprite Attributes
            let use_obp1 = (attributes & 0x10) != 0;
            let x_flip = (attributes & 0x20) != 0;
            let y_flip = (attributes & 0x40) != 0;
            let behind_bg = (attributes & 0x80) != 0;

            let palette = if use_obp1 { self.obp1 } else { self.obp0 };

            // Calculate vertical offset
            let mut tile_y_offset = self.ly.wrapping_sub(sprite_y);
            if y_flip {
                tile_y_offset = sprite_height - 1 - tile_y_offset;
            }

            // 8x16 handling
            let effective_tile_index = if sprite_height == 16 {
                tile_index & 0xFE
            } else {
                tile_index
            };

            // Sprites always use 0x8000 addressing mode (VRAM Offset 0x0000).
            let tile_data_vram_offset = effective_tile_index as u16 * 16;
            let line_offset = tile_y_offset as u16 * 2;

            let data1 = self.internal_read_vram(tile_data_vram_offset + line_offset);
            let data2 = self.internal_read_vram(tile_data_vram_offset + line_offset + 1);

            // Render pixel by pixel
            for x_offset in 0..8 {
                let screen_x_u8 = sprite_x.wrapping_add(x_offset);
                if screen_x_u8 >= SCREEN_WIDTH as u8 {
                    continue;
                }
                let screen_x = screen_x_u8 as usize;

                // Determine the bit to read based on X flip
                let bit = if x_flip { x_offset } else { 7 - x_offset };
                let color_index = (((data2 >> bit) & 1) << 1) | ((data1 >> bit) & 1);

                // Color index 0 is transparent
                if color_index == 0 {
                    continue;
                }

                // Check OBJ-to-BG Priority
                if behind_bg && bg_win_indices[screen_x] != 0 {
                    continue;
                }

                let color = self.get_palette_color(palette, color_index);
                self.set_pixel(screen_x, y, color);
            }
        }
    }

    // --- Helpers ---

    // Defines the DMG color palette (Classic Green Tones)
    fn get_rgb_color(shade: u8) -> u32 {
        match shade {
            0 => 0x9BBC0F, // Lightest Green
            1 => 0x8BAC0F, // Light Green
            2 => 0x306230, // Dark Green
            3 => 0x0F380F, // Darkest Green
            _ => unreachable!(),
        }
    }

    fn get_palette_color(&self, palette: u8, index: u8) -> u32 {
        let shade = (palette >> (index * 2)) & 0x03;
        Self::get_rgb_color(shade)
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < SCREEN_WIDTH && y < SCREEN_HEIGHT {
             self.framebuffer[y * SCREEN_WIDTH + x] = color;
        }
    }

    // Internal memory access (expects offset)
    fn internal_read_vram(&self, offset: u16) -> u8 {
        self.vram[offset as usize]
    }

    fn internal_read_oam(&self, offset: u16) -> u8 {
        self.oam[offset as usize]
    }

    // --- Public Memory Interface (For the Memory Bus/MMU) ---

    fn is_vram_accessible(&self) -> bool {
        !self.is_lcd_enabled() || self.mode != PpuMode::PixelTransfer
    }

    fn is_oam_accessible(&self) -> bool {
        !self.is_lcd_enabled() || (self.mode != PpuMode::OamSearch && self.mode != PpuMode::PixelTransfer)
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9FFF => {
                if self.is_vram_accessible() {
                    self.internal_read_vram(address - 0x8000)
                } else {
                    0xFF
                }
            }
            0xFE00..=0xFE9F => {
                if self.is_oam_accessible() {
                    self.internal_read_oam(address - 0xFE00)
                } else {
                    0xFF
                }
            }
            0xFF40 => self.lcdc,
            0xFF41 => self.stat | 0x80,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => 0xFF,
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0x9FFF => {
                if self.is_vram_accessible() {
                    self.vram[(address & 0x1FFF) as usize] = value;
                }
            }
            0xFE00..=0xFE9F => {
                if self.is_oam_accessible() {
                    self.oam[(address - 0xFE00) as usize] = value;
                }
            }
            0xFF40 => { // LCDC
                let old_lcdc = self.lcdc;
                self.lcdc = value;

                let old_enabled = (old_lcdc & 0x80) != 0;
                let new_enabled = self.is_lcd_enabled();

                if old_enabled && !new_enabled {
                    self.handle_lcd_disable();
                } else if !old_enabled && new_enabled {
                    self.handle_lcd_enable();
                }
                // Note: Changes to LCDC.5 (Window Enable) take effect immediately during rendering,
                // which is handled by the logic inside render_bg_and_window.
            },
            0xFF41 => { // STAT
                self.stat = (value & 0x78) | (self.stat & 0x07);
                self.update_stat_irq();
            }
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => (), // LY is read-only
            0xFF45 => { // LYC
                self.lyc = value;
                if self.is_lcd_enabled() {
                    self.check_lyc();
                }
            }
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => { // WY
                self.wy = value;
                // Check WY condition immediately if LCD is enabled (handles mid-frame updates to WY)
                if self.is_lcd_enabled() {
                    self.check_wy_trigger();
                }
            }
            0xFF4B => self.wx = value,
            _ => (),
        }
    }
    
    // DMA Transfer
    pub fn perform_dma_transfer(&mut self, data: &[u8]) {
        if data.len() == OAM_SIZE {
            self.oam.copy_from_slice(data);
        }
    }
}