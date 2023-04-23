use crate::boot::BOOTROM;
use crate::cartridge;
use crate::cartridge::Cartridge;

use self::memory::Memory;
use super::cpu::interrupts::InterruptFlags;
use log::{info, warn};
use std::io;
use std::io::prelude::*;
use std::{cell::RefCell, rc::Rc};
pub mod memory;

/// MMU is the Memory Management Unit. While the GameBoy did not have an actual
/// MMU, it makes sense for our emulator. The GameBoy uses Memory Mapping to talk to
/// various subsystems. The MMU will be responsible for handling that mapping and will
/// be the only thing to actually access the memory directly.
///
/// The Game Boy has a 16-bit address bus, which is used to address ROM, RAM, and I/O.
///
/// Start    End    Description                        Notes
/// 0000    3FFF    16 KiB ROM bank 00                 From cartridge, usually a fixed bank
/// 4000    7FFF    16 KiB ROM Bank 01~NN              From cartridge, switchable bank via mapper (if any)
/// 8000    9FFF    8 KiB Video RAM (VRAM)             In CGB mode, switchable bank 0/1
/// A000    BFFF    8 KiB External RAM                 From cartridge, switchable bank if any
/// C000    CFFF    4 KiB Work RAM (WRAM)
/// D000    DFFF    4 KiB Work RAM (WRAM)              In CGB mode, switchable bank 1~7
/// E000    FDFF    Mirror of C000~DDFF (ECHO RAM)     Nintendo says use of this area is prohibited.
/// FE00    FE9F    Sprite attribute table (OAM)
/// FEA0    FEFF    Not Usable                         Nintendo says use of this area is prohibited
/// FF00    FF7F    I/O Registers                      https://gbdev.io/pandocs/Hardware_Reg_List.html
/// FF80    FFFE    High RAM (HRAM)
/// FFFF    FFFF    Interrupt Enable register (IE)
///
/// https://gbdev.io/pandocs/Memory_Map.html
pub struct Mmu {
    /// ROM Bank 00 - From cartridge, usually a fixed bank.
    //rom0: [u8; 0x3FFF + 1],

    /// ROM Bank 01~NN - From cartridge, switchable bank via mapper (if any).
    //romx: [u8; (0x7FFF - 0x4000) + 1],

    // Cartridge ROM Banks
    cartridge: Box<dyn Cartridge>,

    /// Video RAM (VRAM) - In CGB mode, switchable bank 0/1.
    vram: [u8; (0x9FFF - 0x8000) + 1],

    /// External RAM (SRAM) - From cartridge, switchable bank (if any).
    //sram: [u8; (0xBFFF - 0xA000) + 1],

    /// Work RAM Bank 00 (WRAM).
    wram0: [u8; (0xCFFF - 0xC000) + 1],

    /// Work RAM Bank 01~07 (WRAMX) - In CGB mode, switchable bank.
    wramx: [u8; (0xDFFF - 0xD000) + 1],

    /// Sprite attribute table (OAM).
    oam: [u8; (0xFE9F - 0xFE00) + 1],

    /// I/O Registers.
    io: [u8; (0xFF7F - 0xFF00) + 1],

    /// Interrupt Flags (IF).
    if_: Rc<RefCell<InterruptFlags>>,

    /// High RAM (HRAM).
    hram: [u8; (0xFFFE - 0xFF80) + 1],

    ///Interrupt Enable register (IE)
    ie: u8,
}

impl Mmu {
    pub fn new(rom_path: String) -> Self {
        let cartridge = cartridge::new(rom_path);
        let interrupt_flags = Rc::new(RefCell::new(InterruptFlags::new()));
        Self {
            cartridge,
            vram: [0x00; (0x9FFF - 0x8000) + 1],
            wram0: [0x00; (0xCFFF - 0xC000) + 1],
            wramx: [0x00; (0xDFFF - 0xD000) + 1],
            oam: [0x00; (0xFE9F - 0xFE00) + 1],
            io: [0x00; (0xFF7F - 0xFF00) + 1],
            if_: interrupt_flags,
            hram: [0x00; (0xFFFE - 0xFF80) + 1],
            ie: 0x00,
        }
    }
}

impl Memory for Mmu {
    /// Read a byte (u8) from memory.
    fn read8(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => {
                // Should we read from Boot ROM?
                if addr <= 0xFF {
                    // Is the Boot ROM enabled?
                    if self.io[0x50] == 0x00 {
                        // Yes, read from Boot ROM.
                        info!("Reading from Boot ROM: {:04X}", addr);
                        return BOOTROM[addr as usize];
                    } else {
                        // No, read from ROM0.
                        info!("Reading from ROM0: {:04X}", addr);
                        return self.cartridge.read8(addr);
                    }
                }
                self.cartridge.read8(addr)
            }
            0x4000..=0x7FFF => self.cartridge.read8(addr),
            0x8000..=0x9FFF => self.vram[addr as usize - 0x8000],
            0xA000..=0xBFFF => self.cartridge.read8(addr),
            0xC000..=0xCFFF | 0xE000..=0xEFFF => self.wram0[addr as usize & 0x0FFF],
            0xD000..=0xDFFF | 0xF000..=0xFDFF => self.wramx[addr as usize & 0x0FFF],
            0xFE00..=0xFE9F => self.oam[addr as usize - 0xFE00],
            0xFF00..=0xFF7F => {
                match addr {
                    // TODO: Implement the rest of the IO registers.
                    0xFF0F => {
                        // Interrupt Flags
                        self.if_.borrow().get_raw()
                    }

                    // Stub LY, for testing.
                    0xFF44 => 0x90,

                    _ => self.io[addr as usize - 0xFF00],
                }
            }
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80],
            0xFFFF => self.ie,
            _ => {
                warn!("Attempt to read prohibited area of memory, {:#02x}.", addr);
                // 0xFEA0 - 0xFEFF is prohibited.
                // DMG will return 0x00.
                // https://gbdev.io/pandocs/Memory_Map.html
                0x00
            }
        }
    }

    /// Write a byte (u8) to memory.
    fn write8(&mut self, addr: u16, val: u8) {
        info!(
            "MMU Write8 val --> [addr]: {:#02x} --> [{:#02x}]",
            val, addr
        );
        match addr {
            0x0000..=0x3FFF => self.cartridge.write8(addr, val),
            0x4000..=0x7FFF => self.cartridge.write8(addr, val),
            0x8000..=0x9FFF => self.vram[addr as usize - 0x8000] = val,
            0xA000..=0xBFFF => self.cartridge.write8(addr, val),
            0xC000..=0xCFFF | 0xE000..=0xEFFF => self.wram0[addr as usize & 0x0FFF] = val,
            0xD000..=0xDFFF | 0xF000..=0xFDFF => self.wramx[addr as usize & 0x0FFF] = val,
            0xFE00..=0xFE9F => self.oam[addr as usize - 0xFE00] = val,
            0xFF00..=0xFF7F => {
                match addr {
                    //TODO: Implement the rest of the IO registers.
                    0xFF0F => {
                        // Interrupt Flags
                        self.if_.borrow_mut().set_raw(val);
                    }
                    // Intercept Serial writes, and output to stdout.
                    0xFF01 => {
                        // Output serial data, and flush stdout.
                        print!("{}", val as char);
                        io::stdout().flush().unwrap();
                        self.io[addr as usize - 0xFF00] = val;
                    }
                    _ => self.io[addr as usize - 0xFF00] = val,
                }
            }
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80] = val,
            0xFFFF => self.ie = val,
            _ => {
                warn!("Attempt to write prohibited area of memory, {:#02x}.", addr);
            }
        }
    }

    /// Read a word (u16) from memory
    fn read16(&self, addr: u16) -> u16 {
        u16::from(self.read8(addr)) | (u16::from(self.read8(addr + 1)) << 8)
        /*let lo = self.read8(addr);
        let hi = self.read8(addr + 1);
        u16::from_le_bytes([lo, hi])*/
    }

    /// Write a word (u16) to memory
    fn write16(&mut self, addr: u16, val: u16) {
        info!(
            "MMU Write16 val --> [addr]: {:#02x} --> [{:#02x}]",
            val, addr
        );
        self.write8(addr, (val & 0xFF) as u8);
        self.write8(addr + 1, (val >> 8) as u8);
    }
}
