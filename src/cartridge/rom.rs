use log::info;

use super::{Cartridge, CartridgeHeader};
use crate::mmu::memory::Memory;
use std::cell::RefCell;
use std::rc::Rc;

impl Cartridge {
    /// Load ROM from file into memory, and parse the header.
    pub fn load(path: String, mem: Rc<RefCell<dyn Memory>>) -> Self {
        let mut header = CartridgeHeader::new();

        // Load ROM into memory while parsing the header
        let rom = std::fs::read(path.clone()).unwrap();
        for (addr, val) in rom.iter().enumerate() {
            if addr < header.entry_point.len() {
                header.entry_point[addr] = *val;
            } else if addr < header.entry_point.len() + header.nintendo_logo.len() {
                header.nintendo_logo[addr - header.entry_point.len()] = *val;
            } else if addr
                < header.entry_point.len() + header.nintendo_logo.len() + header.title.len()
            {
                header.title[addr - header.entry_point.len() - header.nintendo_logo.len()] = *val;
            } else if addr
                < header.entry_point.len()
                    + header.nintendo_logo.len()
                    + header.title.len()
                    + header.manufacturer_code.len()
            {
                header.manufacturer_code[addr
                    - header.entry_point.len()
                    - header.nintendo_logo.len()
                    - header.title.len()] = *val;
            } else if addr
                == header.entry_point.len()
                    + header.nintendo_logo.len()
                    + header.title.len()
                    + header.manufacturer_code.len()
            {
                header.cgb_flag = *val;
            } else if addr
                == header.entry_point.len()
                    + header.nintendo_logo.len()
                    + header.title.len()
                    + header.manufacturer_code.len()
                    + 1
            {
                header.new_licensee_code[0] = *val;
            } else if addr
                == header.entry_point.len()
                    + header.nintendo_logo.len()
                    + header.title.len()
                    + header.manufacturer_code.len()
                    + 2
            {
                header.new_licensee_code[1] = *val;
            } else if addr
                == header.entry_point.len()
                    + header.nintendo_logo.len()
                    + header.title.len()
                    + header.manufacturer_code.len()
                    + 3
            {
                header.sgb_flag = *val;
            } else if addr
                == header.entry_point.len()
                    + header.nintendo_logo.len()
                    + header.title.len()
                    + header.manufacturer_code.len()
                    + 4
            {
                header.cartridge_type = *val;
            } else if addr
                == header.entry_point.len()
                    + header.nintendo_logo.len()
                    + header.title.len()
                    + header.manufacturer_code.len()
                    + 5
            {
                header.rom_size = *val;
            } else if addr
                == header.entry_point.len()
                    + header.nintendo_logo.len()
                    + header.title.len()
                    + header.manufacturer_code.len()
                    + 6
            {
                header.ram_size = *val;
            } else if addr
                == header.entry_point.len()
                    + header.nintendo_logo.len()
                    + header.title.len()
                    + header.manufacturer_code.len()
                    + 7
            {
                header.destination_code = *val;
            } else if addr
                == header.entry_point.len()
                    + header.nintendo_logo.len()
                    + header.title.len()
                    + header.manufacturer_code.len()
                    + 8
            {
                header.old_licensee_code = *val;
            } else if addr
                == header.entry_point.len()
                    + header.nintendo_logo.len()
                    + header.title.len()
                    + header.manufacturer_code.len()
                    + 9
            {
                header.mask_rom_version_number = *val;
            } else if addr
                == header.entry_point.len()
                    + header.nintendo_logo.len()
                    + header.title.len()
                    + header.manufacturer_code.len()
                    + 10
            {
                header.header_checksum = *val;
            } else if addr
                < header.entry_point.len()
                    + header.nintendo_logo.len()
                    + header.title.len()
                    + header.manufacturer_code.len()
                    + 12
            {
                header.global_checksum[addr
                    - header.entry_point.len()
                    - header.nintendo_logo.len()
                    - header.title.len()
                    - header.manufacturer_code.len()
                    - 11] = *val;
            }

            mem.borrow_mut().write8(addr as u16, *val);
        }

        info!("Loaded ROM: {}", path);
        info!(
            "Title: {}",
            String::from_utf8(header.title.to_vec()).unwrap()
        );
        info!("Cartridge type: {}", header.cartridge_type);
        info!("ROM size: {}", header.rom_size);
        info!("RAM size: {}", header.ram_size);

        Self { path, header }
    }
}

impl CartridgeHeader {
    pub fn new() -> Self {
        Self {
            entry_point: [0; 4],
            nintendo_logo: [0; 48],
            title: [0; 16],
            manufacturer_code: [0; 4],
            cgb_flag: 0,
            new_licensee_code: [0; 2],
            sgb_flag: 0,
            cartridge_type: 0,
            rom_size: 0,
            ram_size: 0,
            destination_code: 0,
            old_licensee_code: 0,
            mask_rom_version_number: 0,
            header_checksum: 0,
            global_checksum: [0; 2],
        }
    }
}
