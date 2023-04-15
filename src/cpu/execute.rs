use super::{
    opcodes,
    registers::{Reg16, Reg8},
    Cpu,
};
use log::info;
use std::collections::HashMap;

impl Cpu {
    /// Executes a CPU operation, returns the number of cycles
    pub(super) fn op_execute(&mut self, op: u8) -> (u8, u32) {
        let opcodes: &HashMap<u8, &'static opcodes::OpCode> = &opcodes::OPCODES_MAP;
        let opcode = opcodes.get(&op).unwrap();

        info!("{:#02x} {}", opcode.op, &opcode.mnemonic);

        match op {
            // 0x00 - NOP - No operation
            0x00 => {}

            // LD r8, d8
            // 0x06 - LD B, d8 - Load immediate 8-bit value into register B
            // 0x0E - LD C, d8 - Load immediate 8-bit value into register C
            // 0x16 - LD D, d8 - Load immediate 8-bit value into register D
            // 0x1E - LD E, d8 - Load immediate 8-bit value into register E
            // 0x26 - LD H, d8 - Load immediate 8-bit value into register H
            // 0x2E - LD L, d8 - Load immediate 8-bit value into register L
            // 0x36 - LD (HL), d8 - Load immediate 8-bit value into memory at address HL
            // 0x3E - LD A, d8 - Load immediate 8-bit value into register A
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
                let val = self.imm8();
                match op {
                    0x06 => self.ldr8(Reg8::B, val),
                    0x0E => self.ldr8(Reg8::C, val),
                    0x16 => self.ldr8(Reg8::D, val),
                    0x1E => self.ldr8(Reg8::E, val),
                    0x26 => self.ldr8(Reg8::H, val),
                    0x2E => self.ldr8(Reg8::L, val),
                    0x36 => self.ld8(self.reg.read16(Reg16::HL), val),
                    0x3E => self.ldr8(Reg8::A, val),
                    _ => {}
                }
            }

            // LD r16, d16
            // 0x01 - LD BC, d16 - Load 16-bit immediate value d16 into register BC
            // 0x11 - LD DE, d16 - Load 16-bit immediate value d16 into register DE
            // 0x21 - LD HL, d16 - Load 16-bit immediate value d16 into register HL
            // 0x31 - LD SP, d16 - Load 16-bit immediate value d16 into register SP
            0x01 | 0x11 | 0x21 | 0x31 => {
                let val = self.imm16();
                match op {
                    0x01 => self.ldr16(Reg16::BC, val),
                    0x11 => self.ldr16(Reg16::DE, val),
                    0x21 => self.ldr16(Reg16::HL, val),
                    0x31 => self.ldr16(Reg16::SP, val),
                    _ => {}
                }
            }

            // LD (r16), A
            // 0x02 - LD (BC), A - Load A into memory at address BC
            // 0x12 - LD (DE), A - Load A into memory at address DE
            // 0x22 - LD (HL+), A - Load A into memory at address HL, then increment HL
            // 0x32 - LD (HL-), A - Load A into memory at address HL, then decrement HL
            0x02 | 0x12 | 0x22 | 0x32 => {
                let a = self.reg.read8(Reg8::A);
                match op {
                    0x02 => self.ld8(self.reg.read16(Reg16::BC), a),
                    0x12 => self.ld8(self.reg.read16(Reg16::DE), a),
                    0x22 | 0x32 => {
                        self.ld8(self.reg.read16(Reg16::HL), self.reg.read8(Reg8::A));
                        match op {
                            0x22 => self.reg.write16(Reg16::HL, self.reg.read16(Reg16::HL) + 1),
                            0x32 => self.reg.write16(Reg16::HL, self.reg.read16(Reg16::HL) - 1),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            // LD A, (r16)
            // 0x0A - LD A, (BC) - Load memory at address BC into register A
            // 0x1A - LD A, (DE) - Load memory at address DE into register A
            // 0x2A - LD A, (HL+) - Load memory at address HL into register A, then increment HL
            // 0x3A - LD A, (HL-) - Load memory at address HL into register A, then decrement HL
            0x0A | 0x1A | 0x2A | 0x3A => {
                let val = match op {
                    0x0A => self.mem.borrow().read8(self.reg.read16(Reg16::BC)),
                    0x1A => self.mem.borrow().read8(self.reg.read16(Reg16::DE)),
                    0x2A | 0x3A => self.mem.borrow().read8(self.reg.read16(Reg16::HL)),
                    _ => 0x00,
                };
                match op {
                    0x2A => self.reg.write16(Reg16::HL, self.reg.read16(Reg16::HL) + 1),
                    0x3A => self.reg.write16(Reg16::HL, self.reg.read16(Reg16::HL) - 1),
                    _ => {}
                }
                self.ldr8(Reg8::A, val);
            }

            // LD B, r8
            // 0x40 - LD B, B - Load register B into register B
            // 0x41 - LD B, C - Load register C into register B
            // 0x42 - LD B, D - Load register D into register B
            // 0x43 - LD B, E - Load register E into register B
            // 0x44 - LD B, H - Load register H into register B
            // 0x45 - LD B, L - Load register L into register B
            // 0x47 - LD B, A - Load register A into register B
            0x40 | 0x41 | 0x42 | 0x43 | 0x44 | 0x45 | 0x47 => {
                let val = match op {
                    0x40 => self.reg.read8(Reg8::B),
                    0x41 => self.reg.read8(Reg8::C),
                    0x42 => self.reg.read8(Reg8::D),
                    0x43 => self.reg.read8(Reg8::E),
                    0x44 => self.reg.read8(Reg8::H),
                    0x45 => self.reg.read8(Reg8::L),
                    0x47 => self.reg.read8(Reg8::A),
                    _ => 0x00,
                };
                self.ldr8(Reg8::B, val);
            }

            // LD C, r8
            // 0x48 - LD C, B - Load register B into register C
            // 0x49 - LD C, C - Load register C into register C
            // 0x4A - LD C, D - Load register D into register C
            // 0x4B - LD C, E - Load register E into register C
            // 0x4C - LD C, H - Load register H into register C
            // 0x4D - LD C, L - Load register L into register C
            // 0x4F - LD C, A - Load register A into register C
            0x48 | 0x49 | 0x4A | 0x4B | 0x4C | 0x4D | 0x4F => {
                let val = match op {
                    0x48 => self.reg.read8(Reg8::B),
                    0x49 => self.reg.read8(Reg8::C),
                    0x4A => self.reg.read8(Reg8::D),
                    0x4B => self.reg.read8(Reg8::E),
                    0x4C => self.reg.read8(Reg8::H),
                    0x4D => self.reg.read8(Reg8::L),
                    0x4F => self.reg.read8(Reg8::A),
                    _ => 0x00,
                };
                self.ldr8(Reg8::C, val);
            }

            // LD D, r8
            // 0x50 - LD D, B - Load register B into register D
            // 0x51 - LD D, C - Load register C into register D
            // 0x52 - LD D, D - Load register D into register D
            // 0x53 - LD D, E - Load register E into register D
            // 0x54 - LD D, H - Load register H into register D
            // 0x55 - LD D, L - Load register L into register D
            // 0x57 - LD D, A - Load register A into register D
            0x50 | 0x51 | 0x52 | 0x53 | 0x54 | 0x55 | 0x57 => {
                let val = match op {
                    0x50 => self.reg.read8(Reg8::B),
                    0x51 => self.reg.read8(Reg8::C),
                    0x52 => self.reg.read8(Reg8::D),
                    0x53 => self.reg.read8(Reg8::E),
                    0x54 => self.reg.read8(Reg8::H),
                    0x55 => self.reg.read8(Reg8::L),
                    0x57 => self.reg.read8(Reg8::A),
                    _ => 0x00,
                };
                self.ldr8(Reg8::D, val);
            }

            // LD r8, (HL)
            // 0x46 - LD B, (HL) - Load memory at address HL into register B
            // 0x4E - LD C, (HL) - Load memory at address HL into register C
            // 0x56 - LD D, (HL) - Load memory at address HL into register D
            // 0x5E - LD E, (HL) - Load memory at address HL into register E
            // 0x66 - LD H, (HL) - Load memory at address HL into register H
            // 0x6E - LD L, (HL) - Load memory at address HL into register L
            // 0x7E - LD A, (HL) - Load memory at address HL into register A
            0x46 | 0x4E | 0x56 | 0x5E | 0x66 | 0x6E | 0x7E => {
                let val = self.mem.borrow().read8(self.reg.read16(Reg16::HL));
                match op {
                    0x46 => self.ldr8(Reg8::B, val),
                    0x4E => self.ldr8(Reg8::C, val),
                    0x56 => self.ldr8(Reg8::D, val),
                    0x5E => self.ldr8(Reg8::E, val),
                    0x66 => self.ldr8(Reg8::H, val),
                    0x6E => self.ldr8(Reg8::L, val),
                    0x7E => self.ldr8(Reg8::A, val),
                    _ => {}
                }
            }

            _ => {
                todo!("opcode: {:#02x}.", op);
            }
        }

        (opcode.length, opcode.cycles)
    }
}

impl Cpu {
    /// Fetch the immediate byte (u8).
    /// NOTE: incrementing the PC is not handled.
    fn imm8(&mut self) -> u8 {
        self.mem.borrow().read8(self.reg.read16(Reg16::PC) + 1)
    }

    /// Fetch the immediate word (u16).
    /// NOTE: incrementing the PC is not handled.
    fn imm16(&mut self) -> u16 {
        self.mem.borrow().read16(self.reg.read16(Reg16::PC) + 1)
    }

    /// 8-bit load operation.
    /// Load an 8-bit value (val) into the 16-bit address (dst).
    fn ld8(&mut self, dst: u16, val: u8) {
        self.mem.borrow_mut().write8(dst, val);
    }

    /// 8-bit register load operation.
    /// Load an 8-bit value (val) into the 8-bit register (dst).
    fn ldr8(&mut self, dst: Reg8, val: u8) {
        self.reg.write8(dst, val);
    }

    /// 16-bit load operation.
    /// Load a 16-bit value (val) into the 16-bit address (dst).
    fn ld16(&mut self, dst: u16, val: u16) {
        self.mem.borrow_mut().write16(dst, val);
    }

    /// 16-bit load register operation.
    /// Load a 16-bit value (val) into the 16-bit register (dst).
    fn ldr16(&mut self, dst: Reg16, val: u16) {
        self.reg.write16(dst, val);
    }
}
