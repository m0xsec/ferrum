use bitflags::bitflags;
use std::fmt;

bitflags!(
    /// The Gameboy Flags Register consists of the following bits:
    ///
    /// Bit: 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
    /// Val: Z | N | H | C | 0 | 0 | 0 | 0 |
    ///
    /// Bit 0 - Unused (always 0)
    /// Bit 1 - Unused (always 0)
    /// Bit 2 - Unused (always 0)
    /// Bit 3 - Unused (always 0)
    /// Bit 4 - Carry Flag (N)
    /// Bit 5 - Half Carry Flag (H)
    /// Bit 6 - Subtract Flag (N)
    /// Bit 7 - Zero Flag (Z)
    ///
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

/// Gameboy CPU Registers
/// Most registers are 8bit, but some can be used as 16bit.
/// AF, BC, DE, HL are such registers.
///
/// A - Accumulator (Used for arithmetic operations)
/// F - Flags
/// B - B General Purpose (Can be used as 16 bit register - BC)
/// C - C General Purpose (Can be used as 16 bit register - BC)
/// D - D General Purpose (Can be used as 16 bit register - DE)
/// E - E General Purpose (Can be used as 16 bit register - DE)
/// H - H General Purpose (Can be used as 16 bit register - HL)
/// L - L General Purpose (Can be used as 16 bit register - HL)
/// SP - Stack Pointer
/// PC - Program Counter
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

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\nA:{:02x}\tF:{:04b}\tB:{:02x}\tC:{:02x}\n\
             D:{:02x}\tE:{:02x}\tH:{:02x}\tL:{:02x}\n\
             PC:{:04x}\tSP:{:04x}",
            self.pc, self.sp, self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l
        )
    }
}
