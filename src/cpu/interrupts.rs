/// FF0F - IF - Interrupt Flag (R/W)
/// Bit 0: V-Blank  Interrupt Request (INT 40h)  (1=Request)
/// Bit 1: LCD STAT Interrupt Request (INT 48h)  (1=Request)
/// Bit 2: Timer    Interrupt Request (INT 50h)  (1=Request)
/// Bit 3: Serial   Interrupt Request (INT 58h)  (1=Request)
/// Bit 4: Joypad   Interrupt Request (INT 60h)  (1=Request)
/*pub enum Flags {
    VBlank,
    LCDStat,
    Timer,
    Serial,
    Joypad,
}*/

pub struct InterruptFlags {
    /// Interrupt Flag Register (IF)
    if_: u8,
}

impl InterruptFlags {
    /// Create a new InterruptFlags struct.
    pub fn new() -> Self {
        Self { if_: 0 }
    }

    /*
        /// Set an interrupt flag.
        pub fn set(&mut self, flag: Flags) {
            match flag {
                Flags::VBlank => self.if_ |= 0b0000_0001,
                Flags::LCDStat => self.if_ |= 0b0000_0010,
                Flags::Timer => self.if_ |= 0b0000_0100,
                Flags::Serial => self.if_ |= 0b0000_1000,
                Flags::Joypad => self.if_ |= 0b0001_0000,
            }
        }

    */
    /// Set the raw value of the IF register.
    pub fn set_raw(&mut self, if_: u8) {
        self.if_ = if_;
    }

    /*
        /// Clear an interrupt flag.
        pub fn clear(&mut self, flag: Flags) {
            match flag {
                Flags::VBlank => self.if_ &= 0b1111_1110,
                Flags::LCDStat => self.if_ &= 0b1111_1101,
                Flags::Timer => self.if_ &= 0b1111_1011,
                Flags::Serial => self.if_ &= 0b1111_0111,
                Flags::Joypad => self.if_ &= 0b1110_1111,
            }
        }

        /// Get the value of an interrupt flag.
        pub fn get(&self, flag: Flags) -> bool {
            match flag {
                Flags::VBlank => self.if_ & 0b0000_0001 != 0,
                Flags::LCDStat => self.if_ & 0b0000_0010 != 0,
                Flags::Timer => self.if_ & 0b0000_0100 != 0,
                Flags::Serial => self.if_ & 0b0000_1000 != 0,
                Flags::Joypad => self.if_ & 0b0001_0000 != 0,
            }
        }
    */
    /// Get the raw value of the IF register.
    pub fn get_raw(&self) -> u8 {
        self.if_
    }
}
