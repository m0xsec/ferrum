use crate::cpu;
use crate::mmu;
use crate::ppu::{SCREEN_HEIGHT, SCREEN_PIXELS, SCREEN_WIDTH};
use log::warn;
use minifb::KeyRepeat;
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
        let render_scale = 2;
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
        let mut buffer: Vec<u32> = vec![0; SCREEN_PIXELS];
        window
            .update_with_buffer(buffer.as_slice(), SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();

        // Emulation loop
        let mut emulate = true;
        while emulate {
            // Stop emulation if window is closed.
            if !window.is_open() {
                emulate = false;
            }

            // Simulate correct CPU speed.
            while ticks < waitticks {
                self.cpu.dump_registers();
                ticks += self.cpu.cycle();
            }

            // Is the PPU ready to render?
            let updated = self.mmu.borrow_mut().ppu_updated();
            if updated {
                // Update window buffer
                let viewport = self.mmu.borrow_mut().ppu_get_viewport().clone();
                for y in 0..SCREEN_HEIGHT {
                    for x in 0..SCREEN_WIDTH {
                        let pixel = viewport[y][x];
                        buffer[y * SCREEN_WIDTH + x] = pixel;
                    }
                }

                window
                    .update_with_buffer(buffer.as_slice(), SCREEN_WIDTH, SCREEN_HEIGHT)
                    .unwrap();
            }

            // Handle keyboard input.
            // TODO: Handle Gameboy Joypad input.
            window
                .get_keys_pressed(KeyRepeat::No)
                .iter()
                .for_each(|key| match key {
                    Key::Escape => emulate = false,
                    Key::Space => println!("hemlo <3"),
                    _ => (),
                });

            // Maintain correct CPU speed.
            ticks -= waitticks;
            sleep(Duration::from_millis(16));
        }
        // TODO: Handle emulation exit, such as saving RAM to file...
        println!("\nkthxbai <3");
    }
}
