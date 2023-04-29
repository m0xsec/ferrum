use crate::cpu;
use crate::mmu;
use crate::ppu::SCREEN_PIXELS;
use crate::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};
use log::warn;
use minifb::{Key, Window, WindowOptions};
use std::cell::RefCell;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

/// The GameBoy DMG-01 (non-color).
pub struct GameBoy {
    /// The heart of the Gameboy, the CPU.
    /// The CPU is responsible for decoding and executing instructions.
    /// The DMG-01 had a Sharp LR35902 CPU (speculated to be a SM83 core), which is a hybrid of the Z80 and the 8080.
    cpu: cpu::Cpu,

    /// The DMG-01 didn't have an actual Memory Management Unit (MMU), but it had a memory-mapped I/O system with a single RAM chip.
    /// To make emulation easier, we will define a MMU.
    /// The MMU is responsible for mapping memory addresses to actual memory locations.
    mmu: Rc<RefCell<mmu::Mmu>>,
}

impl GameBoy {
    /// Initialize Gameboy Audio Hardware (APU)
    fn init_audio(&mut self) {
        // TODO: Look at using cpal for audio output, spin up a thread to handle audio, etc.
        warn!("Audio is not implemented yet.");
    }
}
impl GameBoy {
    /// Initialize Gameboy Hardware
    pub fn power_on(rom_path: String) -> Self {
        let mmu = Rc::new(RefCell::new(mmu::Mmu::new(rom_path)));
        let cpu = cpu::Cpu::power_on(mmu.clone());

        Self { cpu, mmu }
    }

    /// Run Gameboy emulation
    pub fn run(&mut self) {
        warn!("Emulation loop is a work in progress, no threading or event handling.");

        // The Gameboy runs at 4.194304 MHz.
        // 4194304 Hz / 1000 ms * 16 ms = 67108.8
        let waitticks = (4194304f64 / 1000.0 * 16.0).round() as u32;
        let mut ticks = 0;

        // Initialize Audio
        self.init_audio();

        // Setup window for rendering
        let render_scale = 4;
        let option = WindowOptions {
            resize: false,
            scale: match render_scale {
                1 => minifb::Scale::X1,
                2 => minifb::Scale::X2,
                4 => minifb::Scale::X4,
                8 => minifb::Scale::X8,
                _ => panic!("Invalid render scale: {}", render_scale),
            },
            ..Default::default()
        };
        let rom_title = self.mmu.borrow().rom_title();
        let mut window = Window::new(
            format!("ferrum - {}", rom_title).as_str(),
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            option,
        )
        .unwrap();
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        // Initialize window buffer
        window
            .update_with_buffer(
                self.mmu.borrow_mut().ppu_get_viewport().as_slice(),
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
            )
            .unwrap();

        // Emulation loop
        loop {
            // Stop emulation if window is closed.
            if !window.is_open() {
                break;
            }

            // Simulate correct CPU speed.
            while ticks < waitticks {
                self.cpu.dump_registers();
                ticks += self.cpu.cycle();
            }

            // Is the PPU ready to render?
            let updated = self.mmu.borrow_mut().ppu_updated();
            if updated {
                window
                    .update_with_buffer(
                        self.mmu.borrow_mut().ppu_get_viewport().as_slice(),
                        SCREEN_WIDTH,
                        SCREEN_HEIGHT,
                    )
                    .unwrap();
            }

            // Handle keyboard input.
            // TODO: Handle Gameboy Joypad input.
            if window.is_key_down(Key::Escape) {
                break;
            }

            // Maintain correct CPU speed.
            ticks -= waitticks;
            sleep(Duration::from_millis(16));
        }
        // TODO: Handle emulation exit, such as saving RAM to file...
        println!("\nkthxbai <3");
    }
}
