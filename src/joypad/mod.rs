use std::{cell::RefCell, rc::Rc};

use crate::cpu::interrupts::{Flags, InterruptFlags};

/// GameBoy Joypad (P1/JOYP - $FF00)
///
/// The joypad register uses a matrix layout with two select lines.
/// The game writes to bits 4-5 to select which group to read,
/// then reads bits 0-3 for the button states (active low).
///
/// Bit 5 - P15 Select Action buttons    (0=Select)
/// Bit 4 - P14 Select Direction buttons  (0=Select)
/// Bit 3 - P13 Input: Down  or Start     (0=Pressed)
/// Bit 2 - P12 Input: Up    or Select    (0=Pressed)
/// Bit 1 - P11 Input: Left  or B         (0=Pressed)
/// Bit 0 - P10 Input: Right or A         (0=Pressed)
///
/// https://gbdev.io/pandocs/Joypad_Input.html
pub struct Joypad {
    /// The select bits written by the game (bits 4-5 of P1).
    select: u8,

    /// Direction button states (active high internally, inverted on read).
    /// Bit 0: Right, Bit 1: Left, Bit 2: Up, Bit 3: Down
    directions: u8,

    /// Action button states (active high internally, inverted on read).
    /// Bit 0: A, Bit 1: B, Bit 2: Select, Bit 3: Start
    actions: u8,

    /// Reference to interrupt flags for joypad interrupt.
    if_: Rc<RefCell<InterruptFlags>>,
}

impl Joypad {
    pub fn new(if_: Rc<RefCell<InterruptFlags>>) -> Self {
        Self {
            select: 0x30, // Both deselected by default
            directions: 0x00,
            actions: 0x00,
            if_,
        }
    }

    /// Read the joypad register (0xFF00).
    /// Returns the select bits in 4-5, and the active-low button states in 0-3.
    pub fn read(&self) -> u8 {
        let mut result = self.select | 0xC0; // Bits 6-7 always set

        if self.select & 0x10 == 0 {
            // P14 selected: direction buttons
            result |= (!self.directions) & 0x0F;
        }
        if self.select & 0x20 == 0 {
            // P15 selected: action buttons
            result |= (!self.actions) & 0x0F;
        }

        // If neither is selected, low nibble is 0x0F (no buttons pressed)
        if self.select & 0x30 == 0x30 {
            result |= 0x0F;
        }

        result
    }

    /// Write to the joypad register (0xFF00).
    /// Only bits 4-5 are writable (the select lines).
    pub fn write(&mut self, val: u8) {
        self.select = val & 0x30;
    }

    /// Press a button. Fires joypad interrupt if a button transitions from released to pressed.
    pub fn press(&mut self, button: JoypadButton) {
        let was_pressed = self.is_pressed(button);

        match button {
            JoypadButton::Right => self.directions |= 0x01,
            JoypadButton::Left => self.directions |= 0x02,
            JoypadButton::Up => self.directions |= 0x04,
            JoypadButton::Down => self.directions |= 0x08,
            JoypadButton::A => self.actions |= 0x01,
            JoypadButton::B => self.actions |= 0x02,
            JoypadButton::Select => self.actions |= 0x04,
            JoypadButton::Start => self.actions |= 0x08,
        }

        // Request joypad interrupt on high-to-low transition (button press)
        if !was_pressed {
            self.if_.borrow_mut().set(Flags::Joypad);
        }
    }

    /// Release a button.
    pub fn release(&mut self, button: JoypadButton) {
        match button {
            JoypadButton::Right => self.directions &= !0x01,
            JoypadButton::Left => self.directions &= !0x02,
            JoypadButton::Up => self.directions &= !0x04,
            JoypadButton::Down => self.directions &= !0x08,
            JoypadButton::A => self.actions &= !0x01,
            JoypadButton::B => self.actions &= !0x02,
            JoypadButton::Select => self.actions &= !0x04,
            JoypadButton::Start => self.actions &= !0x08,
        }
    }

    /// Check if a button is currently pressed.
    fn is_pressed(&self, button: JoypadButton) -> bool {
        match button {
            JoypadButton::Right => self.directions & 0x01 != 0,
            JoypadButton::Left => self.directions & 0x02 != 0,
            JoypadButton::Up => self.directions & 0x04 != 0,
            JoypadButton::Down => self.directions & 0x08 != 0,
            JoypadButton::A => self.actions & 0x01 != 0,
            JoypadButton::B => self.actions & 0x02 != 0,
            JoypadButton::Select => self.actions & 0x04 != 0,
            JoypadButton::Start => self.actions & 0x08 != 0,
        }
    }
}

/// The eight buttons on the GameBoy.
#[derive(Clone, Copy)]
pub enum JoypadButton {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}
