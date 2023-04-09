use bitflags::bitflags;

bitflags!(
    /// The Flag register consists of the following bits:
    /// Zero Flag (Z) - Set when the result of a math operation is zero, or two values match when using the CP instruction.
    /// Subtract Flag (N) - This bit is set if a subtraction was performed in the last math instruction.
    /// Half Carry Flag (H) - This bit is set if a carry occurred from the lower nibble in the last math operation.
    /// Carry Flag (C) - This bit is set if a carry occurred from the last math operation or if register A is the smaller value when executing the CP instruction.
    struct Flags: u8 {
        const ZERO         = 0b_1000_0000;
        const ADD_SUBTRACT = 0b_0100_0000;
        const HALF_CARRY   = 0b_0010_0000;
        const CARRY        = 0b_0001_0000;
  }
);

/// The Gameboy has eight 8-bit registers, and two 16-bit registers.
/// Some 8-bit registers can be combined to be used as 16-bit registers.
pub struct Registers {
    /// 8 bit registers
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: Flags,
    h: u8,
    l: u8,

    /// 16-bit registers
    /// sp - Stack pointer.
    /// pc - Program counter
    sp: u16,
    pc: u16,
}

/// Reg8 represents an 8-bit register.
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
}

/// Reg16 represents a 16-bit register.
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: Flags::empty(),
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }

    /// Read a 16-bit register value.
    pub fn read16(&self, reg: Reg16) -> u16 {
        match reg {
            Reg16::AF => ((self.a as u16) << 8) | (self.f.bits() as u16),
            Reg16::BC => ((self.b as u16) << 8) | (self.c as u16),
            Reg16::DE => ((self.d as u16) << 8) | (self.e as u16),
            Reg16::HL => ((self.h as u16) << 8) | (self.l as u16),
            Reg16::SP => self.sp,
        }
    }

    /// Write a 16-bit register value.
    pub fn write16(&mut self, reg: Reg16, value: u16) {
        match reg {
            Reg16::AF => {
                self.a = (value >> 8) as u8;
                self.f = Flags::from_bits_truncate(value as u8)
            }
            Reg16::BC => {
                self.b = (value >> 8) as u8;
                self.c = value as u8
            }
            Reg16::DE => {
                self.d = (value >> 8) as u8;
                self.e = value as u8
            }
            Reg16::HL => {
                self.h = (value >> 8) as u8;
                self.l = value as u8
            }
            Reg16::SP => self.sp = value,
        }
    }
}
