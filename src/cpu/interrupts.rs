/// FF0F - IF - Interrupt Flag (R/W)
/// Bit 0: V-Blank  Interrupt Request (INT 40h)  (1=Request)
/// Bit 1: LCD STAT Interrupt Request (INT 48h)  (1=Request)
/// Bit 2: Timer    Interrupt Request (INT 50h)  (1=Request)
/// Bit 3: Serial   Interrupt Request (INT 58h)  (1=Request)
/// Bit 4: Joypad   Interrupt Request (INT 60h)  (1=Request)
pub enum Flags {
    VBlank,
    LCDStat,
    Timer,
    Serial,
    Joypad,
}

pub struct InterruptFlags {
    /// Interrupt Flag Register (IF)
    pub data: u8,
}

impl InterruptFlags {
    /// Create a new InterruptFlags struct.
    pub fn new() -> Self {
        Self { data: 0x00 }
    }

    /// Set the given flag.
    pub fn set(&mut self, flag: Flags) {
        self.data |= 1 << flag as u8;
    }
}
