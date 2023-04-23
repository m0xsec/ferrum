pub mod header;
pub mod mbc;
pub mod mbc1;

use crate::mmu::memory::Memory;

use self::{header::*, mbc::*, mbc1::*};

/// Cartridge represents a Gameboy ROM
pub trait Cartridge: Memory {
    /// Cartridge Tile
    fn title(&self) -> String {
        let mut title = String::new();
        for i in 0x134..0x143 {
            match self.read8(i) {
                0x00 => break,
                _ => title.push(self.read8(i) as char),
            }
        }
        title
    }

    /// Cartridge Type
    fn mbc(&self) -> CartridgeType {
        CartridgeType::try_from(self.read8(0x147)).unwrap()
    }

    /// ROM Size
    fn rom_size(&self) -> RomSize {
        RomSize::try_from(self.read8(0x148)).unwrap()
    }

    /// RAM Size
    fn ram_size(&self) -> RamSize {
        RamSize::try_from(self.read8(0x149)).unwrap()
    }

    /// Destination Code
    fn destination_code(&self) -> DestinationCode {
        DestinationCode::try_from(self.read8(0x14A)).unwrap()
    }

    /// New Licensee Code
    fn new_licensee_code(&self) -> NewLicenseeCode {
        NewLicenseeCode::try_from(
            ((self.read8(0x144) as u16) << 8 | self.read8(0x145) as u16) as u8,
        )
        .unwrap()
    }

    /// Old Licensee Code
    fn old_licensee_code(&self) -> OldLicenseeCode {
        OldLicenseeCode::try_from(self.read8(0x14B)).unwrap()
    }
}

/// Initialize a new Cartridge.
pub fn new(path: String) -> Box<dyn Cartridge> {
    let rom_data = std::fs::read(path.clone()).unwrap();
    let cart: Box<dyn Cartridge> = match CartridgeType::try_from(rom_data[0x147]).unwrap() {
        CartridgeType::RomOnly => Box::new(RomOnly::new(rom_data)),
        CartridgeType::Mbc1 => Box::new(Mbc1::new(rom_data, vec![])),
        //TODO: Implement other cartridge types.
        _ => todo!("Unsupported cartridge type: {:?}", path),
    };

    println!("\nCartridge Info:");
    println!("\tCartridge Title: {}", cart.title());
    println!("\tCartridge Type: {:?}", cart.mbc());
    println!("\tROM Size: {:?}", cart.rom_size());
    println!("\tRAM Size: {:?}", cart.ram_size());
    println!("\tDestination Code: {:?}", cart.destination_code());
    println!("\tNew Licensee Code: {:?}", cart.new_licensee_code());
    println!("\tOld Licensee Code: {:?}\n", cart.old_licensee_code());

    cart
}
