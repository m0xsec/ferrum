use self::memory::Memory;
use log::{info, warn};
pub mod memory;

/// MMU is the Memory Management Unit. While the GameBoy did not have an actual
/// MMU, it makes sense for our emulator. The GameBoy uses Memory Mapping to talk to
/// various subsystems. The MMU will be responsible for handling that mapping and will
/// be the only thing to actually access the memory directly.
///
/// The Game Boy has a 16-bit address bus, which is used to address ROM, RAM, and I/O.
///
/// Start	End		Description						Notes
/// 0000	3FFF	16 KiB ROM bank 00				From cartridge, usually a fixed bank
/// 4000	7FFF	16 KiB ROM Bank 01~NN			From cartridge, switchable bank via mapper (if any)
/// 8000	9FFF	8 KiB Video RAM (VRAM)			In CGB mode, switchable bank 0/1
/// A000	BFFF	8 KiB External RAM				From cartridge, switchable bank if any
/// C000	CFFF	4 KiB Work RAM (WRAM)
/// D000	DFFF	4 KiB Work RAM (WRAM)			In CGB mode, switchable bank 1~7
/// E000	FDFF	Mirror of C000~DDFF (ECHO RAM)	Nintendo says use of this area is prohibited.
/// FE00	FE9F	Sprite attribute table (OAM)
/// FEA0	FEFF	Not Usable						Nintendo says use of this area is prohibited
/// FF00	FF7F	I/O Registers
/// FF80	FFFE	High RAM (HRAM)
/// FFFF	FFFF	Interrupt Enable register (IE)
///
/// https://gbdev.io/pandocs/Memory_Map.html
pub struct MMU {
    /// ROM Bank 00 - From cartridge, usually a fixed bank.
    rom0: [u8; 0x3FFF - 0x0000],

    /// ROM Bank 01~NN - From cartridge, switchable bank via mapper (if any).
    romx: [u8; 0x7FFF - 0x4000],

    /// Video RAM (VRAM) - In CGB mode, switchable bank 0/1.
    vram: [u8; 0x9FFF - 0x8000],

    /// External RAM (SRAM) - From cartridge, switchable bank (if any).
    sram: [u8; 0xBFFF - 0xA000],

    /// Work RAM Bank 00 (WRAM).
    wram0: [u8; 0xCFFF - 0xC000],

    /// Work RAM Bank 01~07 (WRAMX) - In CGB mode, switchable bank.
    wramx: [u8; 0xDFFF - 0xD000],

    /// Sprite attribute table (OAM).
    oam: [u8; 0xFE9F - 0xFE00],

    /// I/O Registers.
    io: [u8; 0xFF7F - 0xFF00],

    /// High RAM (HRAM).
    hram: [u8; 0xFFFE - 0xFF80],

    ///Interrupt Enable register (IE)
    ie: u8,
}

impl MMU {
    pub fn new() -> Self {
        Self {
            rom0: [0x00; 0x3FFF - 0x0000],
            romx: [0x00; 0x7FFF - 0x4000],
            vram: [0x00; 0x9FFF - 0x8000],
            sram: [0x00; 0xBFFF - 0xA000],
            wram0: [0x00; 0xCFFF - 0xC000],
            wramx: [0x00; 0xDFFF - 0xD000],
            oam: [0x00; 0xFE9F - 0xFE00],
            io: [0x00; 0xFF7F - 0xFF00],
            hram: [0x00; 0xFFFE - 0xFF80],
            ie: 0x00,
        }
    }
}

impl Memory for MMU {
    /// Read a value from memory.
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom0[addr as usize],
            0x4000..=0x7FFF => self.romx[addr as usize],
            0x8000..=0x9FFF => self.vram[addr as usize],
            0xA000..=0xBFFF => self.sram[addr as usize],
            0xC000..=0xCFFF | 0xE000..=0xEFFF => self.wram0[addr as usize],
            0xD000..=0xDFFF | 0xF000..=0xFDFF => self.wramx[addr as usize],
            0xFE00..=0xFE9F => self.oam[addr as usize],
            0xFF00..=0xFF7F => self.io[addr as usize],
            0xFF80..=0xFFFE => self.hram[addr as usize],
            0xFFFF => self.ie,
            _ => {
                warn!("Attempt to read prohibited area of memory, {:#02x}.", addr);
                0xFF
            }
        }
    }

    /// Write a value to memory
    fn write(&mut self, addr: u16, val: u8) {
        info!("MMU Write val --> [addr]: {:#02x} --> [{:#02x}]", val, addr);
        match addr {
            0x0000..=0x3FFF => self.rom0[addr as usize] = val,
            0x4000..=0x7FFF => self.romx[addr as usize] = val,
            0x8000..=0x9FFF => self.vram[addr as usize] = val,
            0xA000..=0xBFFF => self.sram[addr as usize] = val,
            0xC000..=0xCFFF | 0xE000..=0xEFFF => self.wram0[addr as usize] = val,
            0xD000..=0xDFFF | 0xF000..=0xFDFF => self.wramx[addr as usize] = val,
            0xFE00..=0xFE9F => self.oam[addr as usize] = val,
            0xFF00..=0xFF7F => self.io[addr as usize] = val,
            0xFF80..=0xFFFE => self.hram[addr as usize] = val,
            0xFFFF => self.ie = val,
            _ => {
                warn!("Attempt to write prohibited area of memory, {:#02x}.", addr);
            }
        }
    }
}
