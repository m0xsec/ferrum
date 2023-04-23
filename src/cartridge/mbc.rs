use super::Cartridge;
use crate::mmu::memory::Memory;

/// No MBC (ROM Only)
pub struct RomOnly {
    rom: Vec<u8>,
}

impl RomOnly {
    pub fn new(rom: Vec<u8>) -> Self {
        Self { rom }
    }
}

impl Memory for RomOnly {
    fn read8(&self, addr: u16) -> u8 {
        self.rom[addr as usize]
    }

    fn write8(&mut self, _: u16, _: u8) {}

    fn read16(&self, addr: u16) -> u16 {
        u16::from(self.read8(addr)) | (u16::from(self.read8(addr + 1)) << 8)
    }

    fn write16(&mut self, _: u16, _: u16) {}
}

impl Cartridge for RomOnly {}

/// Bank Mode (MBC1)
/// MBC1 has two bank modes:
///   ROM Banking Mode (up to 8KByte RAM, 2MByte ROM) (default)
///   RAM Banking Mode (up to 32KByte RAM, 512KByte ROM)
enum BankMode {
    Rom,
    Ram,
}

/// This is the first MBC chip for the gameboy. Any newer MBC chips are working similiar, so that is relative easy to
/// upgrade a program from one MBC chip to another - or even to make it compatible to several different types of MBCs.
/// Note that the memory in range 0000-7FFF is used for both reading from ROM, and for writing to the MBCs Control
/// Registers.
///
/// 0000-3FFF - ROM Bank 00 (Read Only)
/// This area always contains the first 16KBytes of the cartridge ROM.
///
/// 4000-7FFF - ROM Bank 01-7F (Read Only)
/// This area may contain any of the further 16KByte banks of the ROM, allowing to address up to 125 ROM Banks
/// (almost 2MByte). As described below, bank numbers 20h, 40h, and 60h cannot be used, resulting in the odd amount of
/// 125 banks.
///
/// A000-BFFF - RAM Bank 00-03, if any (Read/Write)
/// This area is used to address external RAM in the cartridge (if any). External RAM is often battery buffered,
/// allowing to store game positions or high score tables, even if the gameboy is turned off, or if the cartridge is
/// removed from the gameboy. Available RAM sizes are: 2KByte (at A000-A7FF), 8KByte (at A000-BFFF), and 32KByte (in
/// form of four 8K banks at A000-BFFF).
///
/// 0000-1FFF - RAM Enable (Write Only)
/// Before external RAM can be read or written, it must be enabled by writing to this address space. It is recommended
/// to disable external RAM after accessing it, in order to protect its contents from damage during power down of the
/// gameboy. Usually the following values are used:
///   00h  Disable RAM (default)
///   0Ah  Enable RAM
/// Practically any value with 0Ah in the lower 4 bits enables RAM, and any other value disables RAM.
///
/// 2000-3FFF - ROM Bank Number (Write Only)
/// Writing to this address space selects the lower 5 bits of the ROM Bank Number (in range 01-1Fh). When 00h is written,
/// the MBC translates that to bank 01h also. That doesn't harm so far, because ROM Bank 00h can be always directly
/// accessed by reading from 0000-3FFF. But (when using the register below to specify the upper ROM Bank bits), the same
/// happens for Bank 20h, 40h, and 60h. Any attempt to address these ROM Banks will select Bank 21h, 41h, and 61h
/// instead.
///
/// 4000-5FFF - RAM Bank Number - or - Upper Bits of ROM Bank Number (Write Only)
/// This 2bit register can be used to select a RAM Bank in range from 00-03h, or to specify the upper two bits (Bit 5-6)
/// of the ROM Bank number, depending on the current ROM/RAM Mode. (See below.)
///
/// 6000-7FFF - ROM/RAM Mode Select (Write Only)
/// This 1bit Register selects whether the two bits of the above register should be used as upper two bits of the ROM
/// Bank, or as RAM Bank Number.
///   00h = ROM Banking Mode (up to 8KByte RAM, 2MByte ROM) (default)
///   01h = RAM Banking Mode (up to 32KByte RAM, 512KByte ROM)
/// The program may freely switch between both modes, the only limitation is that only RAM Bank 00h can be used during
/// Mode 0, and only ROM Banks 00-1Fh can be used during Mode 1.
pub struct Mbc1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    bank_mode: BankMode,
    bank: u8,
    ram_enabled: bool,
}

impl Mbc1 {
    pub fn new(rom: Vec<u8>, ram: Vec<u8>) -> Self {
        Self {
            rom,
            ram,
            bank_mode: BankMode::Rom, // Default bank mode is ROM.
            bank: 0x01,
            ram_enabled: false,
        }
    }

    fn rom_bank(&self) -> usize {
        let bank = match self.bank_mode {
            BankMode::Rom => self.bank & 0x7f,
            BankMode::Ram => self.bank & 0x1f,
        };
        bank as usize
    }

    fn ram_bank(&self) -> usize {
        let bank = match self.bank_mode {
            BankMode::Rom => 0x00,
            BankMode::Ram => (self.bank & 0x60) >> 5,
        };
        bank as usize
    }
}

impl Memory for Mbc1 {
    fn read8(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3fff => self.rom[addr as usize],
            0x4000..=0x7fff => {
                let bank = self.rom_bank();
                let offset = addr as usize - 0x4000;
                self.rom[bank * 0x4000 + offset]
            }
            0xa000..=0xbfff => {
                if self.ram_enabled {
                    let bank = self.ram_bank();
                    let offset = addr as usize - 0xa000;
                    self.ram[bank * 0x2000 + offset]
                } else {
                    0x00
                }
            }
            _ => 0x00,
        }
    }

    fn write8(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1fff => {
                self.ram_enabled = val & 0x0f == 0x0a;
            }
            0x2000..=0x3fff => {
                let b = val & 0x1f;
                self.bank = (self.bank & 0x60) | if b == 0x00 { 0x01 } else { b };
            }
            0x4000..=0x5fff => {
                self.bank = self.bank & 0x9f | ((val & 0x03) << 5);
            }
            0x6000..=0x7fff => {
                self.bank_mode = match val {
                    0x00 => BankMode::Rom,
                    0x01 => BankMode::Ram,
                    _ => panic!("Invalid bank mode: {:#04x}", val),
                };
            }
            0xa000..=0xbfff => {
                if self.ram_enabled {
                    let bank = self.ram_bank();
                    let offset = addr as usize - 0xa000;
                    self.ram[bank * 0x2000 + offset] = val;
                }
            }
            _ => {}
        }
    }

    fn read16(&self, addr: u16) -> u16 {
        u16::from(self.read8(addr)) | (u16::from(self.read8(addr + 1)) << 8)
    }

    fn write16(&mut self, addr: u16, val: u16) {
        self.write8(addr, (val & 0xFF) as u8);
        self.write8(addr + 1, (val >> 8) as u8);
    }
}

impl Cartridge for Mbc1 {}
