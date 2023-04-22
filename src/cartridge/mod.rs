pub mod header;

use self::header::*;

/// Cartridge represents a Gameboy ROM
pub struct Cartridge {
    /// File path to ROM file.
    pub path: String,

    /// ROM data.
    pub data: Vec<u8>,
}

impl Cartridge {
    /// Initialize a new Cartridge.
    pub fn new(path: String) -> Self {
        let rom_data = std::fs::read(path.clone()).unwrap();
        Self {
            path,
            data: rom_data,
        }
    }

    /// Cartridge Tile
    pub fn title(&self) -> String {
        let mut title = String::new();
        for i in 0x134..0x143 {
            title.push(self.data[i] as char);
        }
        title
    }

    /// Cartridge Type
    pub fn mbc(&self) -> CartridgeType {
        CartridgeType::try_from(self.data[0x147]).unwrap()
    }

    /// ROM Size
    pub fn rom_size(&self) -> RomSize {
        RomSize::try_from(self.data[0x148]).unwrap()
    }

    /// RAM Size
    pub fn ram_size(&self) -> RamSize {
        RamSize::try_from(self.data[0x149]).unwrap()
    }

    /// Destination Code
    pub fn destination_code(&self) -> DestinationCode {
        DestinationCode::try_from(self.data[0x14A]).unwrap()
    }

    /// New Licensee Code
    pub fn new_licensee_code(&self) -> NewLicenseeCode {
        NewLicenseeCode::try_from(((self.data[0x144] as u16) << 8 | self.data[0x145] as u16) as u8)
            .unwrap()
    }

    /// Old Licensee Code
    pub fn old_licensee_code(&self) -> OldLicenseeCode {
        OldLicenseeCode::try_from(self.data[0x14B]).unwrap()
    }
}
