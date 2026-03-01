use std::{cell::RefCell, rc::Rc};

use super::{fifo::Fifo, OAM_SIZE, VRAM_SIZE};

/// Pixel Fetcher States.
enum FetcherState {
    ReadTileId,
    ReadTileData0,
    ReadTileData1,
    PushToFifo,
}

/// Pixel Fetcher reads the tile data from the VRAM and stores it in the Pixel FIFO.
pub struct Fetcher {
    /// Pixel FIFO.
    pub fifo: Fifo,

    /// Reference to VRAM.
    vram: Rc<RefCell<[u8; VRAM_SIZE]>>,

    /// Reference to OAM.
    oam: Rc<RefCell<[u8; OAM_SIZE]>>,

    /// Fetcher clock cycles counter, for timing.
    ticks: u8,

    /// Current Fetcher state.
    state: FetcherState,

    /// Start address of BG/Window map.
    map_addr: u16,

    /// Start address of BG/Sprite tile data.
    data_addr: u16,

    /// Tile data addressing mode (true = 0x8000 unsigned, false = 0x8800 signed).
    tile_data_unsigned: bool,

    /// Y offset in the tile.
    tile_line: u8,

    /// Tile index of the tile to read in the background map.
    tile_index: u8,

    /// Tile number from the tile map.
    tile_id: u8,

    /// Pixel data for one row of the fetched tile.
    tile_data: [u8; 8],
}

impl Fetcher {
    pub fn new(vram: Rc<RefCell<[u8; VRAM_SIZE]>>, oam: Rc<RefCell<[u8; OAM_SIZE]>>) -> Fetcher {
        Fetcher {
            fifo: Fifo::new(),
            vram,
            oam,
            ticks: 0,
            state: FetcherState::ReadTileId,
            map_addr: 0,
            data_addr: 0,
            tile_data_unsigned: true,
            tile_line: 0,
            tile_index: 0,
            tile_id: 0,
            tile_data: [0; 8],
        }
    }

    /// Start fetching a line of pixels, starting at the given tile address in the background map.
    /// tile_line indicates which row of pixels to fetch from the tile.
    /// tile_data_unsigned: true = 0x8000 method (unsigned), false = 0x8800 method (signed).
    pub fn start(&mut self, map_addr: u16, tile_line: u8, tile_data_unsigned: bool) {
        self.map_addr = map_addr;
        self.tile_line = tile_line;
        self.tile_data_unsigned = tile_data_unsigned;
        self.tile_index = 0;
        self.state = FetcherState::ReadTileId;

        // Clear the FIFO, as it will likely contain stale data from the previous scan line.
        self.fifo.clear();
    }

    /// Tick advances the fetcher state machine by one step.
    pub fn tick(&mut self) {
        // The fetcher should run at half the speed of the PPU
        self.ticks += 1;
        if self.ticks < 2 {
            return;
        }

        // Reset tick counter, and execute the next state.
        self.ticks = 0;

        match self.state {
            FetcherState::ReadTileId => {
                // Read the tile's number from the background map. This will be used
                // in the next states to find the address where the tile's actual pixel
                // data is stored in memory.
                self.tile_id = self.vram.borrow()
                    [(self.map_addr as usize + self.tile_index as usize) - 0x8000];

                self.state = FetcherState::ReadTileData0;
            }
            FetcherState::ReadTileData0 => {
                // Read the first half of the tile's pixel data.
                self.read_tile_line(0);

                self.state = FetcherState::ReadTileData1;
            }
            FetcherState::ReadTileData1 => {
                // Read the second half of the tile's pixel data.
                self.read_tile_line(1);

                self.state = FetcherState::PushToFifo;
            }
            FetcherState::PushToFifo => {
                if self.fifo.size() <= 8 {
                    // We stored pixel bits from least significant (rightmost) to most
                    // (leftmost) in the data array, so we must push them in reverse
                    // order.
                    for i in (0..8).rev() {
                        self.fifo.push(self.tile_data[i]);
                    }

                    // Advance to the next tile in the background map.
                    self.tile_index += 1;
                    self.state = FetcherState::ReadTileId;
                }
            }
        }
    }

    /// Updates the fetcher's pixel buffer with tile data, depending on current state.
    /// Each pixel requires 2 bits of information, which gets read in two separate steps.
    pub fn read_tile_line(&mut self, bit_plane: u8) {
        // A tile's graphical data takes 16 bytes (2 bytes per row of 8 pixels).
        // Two addressing modes exist:
        //   - 0x8000 method (LCDC.4=1): tile_id is unsigned (0-255), base at 0x8000
        //   - 0x8800 method (LCDC.4=0): tile_id is signed (-128..127), base at 0x9000
        let offset = if self.tile_data_unsigned {
            0x8000u16.wrapping_add(self.tile_id as u16 * 16)
        } else {
            // Signed: tile_id 0-127 maps to 0x9000-0x97FF,
            //         tile_id 128-255 maps to 0x8800-0x8FFF
            0x9000u16.wrapping_add((self.tile_id as i8 as i16 * 16) as u16)
        };

        // Then, from that starting offset, we compute the final address to read
        // by finding out which of the 8-pixel rows of the tile we want to display.
        let addr = offset + (self.tile_line as u16 * 2);

        // Finally, read the first or second byte of graphical data depending on
        // what state we're in.
        let pixel_data = self.vram.borrow()[(addr as usize + bit_plane as usize) - 0x8000];
        for bit_pos in 0..8 {
            // Separate each bit fom the data byte we just read. Each of these bits
            // is half of a pixel's color value.
            if bit_plane == 0 {
                // Least significant bit, replace the previous value.
                self.tile_data[bit_pos] = (pixel_data >> bit_pos) & 0x01;
            } else {
                // Most significant bit, combine with the previous value.
                self.tile_data[bit_pos] |= ((pixel_data >> bit_pos) & 0x01) << 1;
            }
        }
    }
}
