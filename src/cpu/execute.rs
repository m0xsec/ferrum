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

            // INC r16
            // 0x03 - INC BC - Increment register BC
            // 0x13 - INC DE - Increment register DE
            // 0x23 - INC HL - Increment register HL
            // 0x33 - INC SP - Increment register SP
            0x03 | 0x13 | 0x23 | 0x33 => match op {
                0x03 => self
                    .reg
                    .write16(Reg16::BC, self.reg.read16(Reg16::BC).wrapping_add(1)),
                0x13 => self
                    .reg
                    .write16(Reg16::DE, self.reg.read16(Reg16::DE).wrapping_add(1)),
                0x23 => self
                    .reg
                    .write16(Reg16::HL, self.reg.read16(Reg16::HL).wrapping_add(1)),
                0x33 => self
                    .reg
                    .write16(Reg16::SP, self.reg.read16(Reg16::SP).wrapping_add(1)),
                _ => {}
            },

            // ADD HL, r16
            // 0x09 - ADD HL, BC - Add register BC to register HL
            // 0x19 - ADD HL, DE - Add register DE to register HL
            // 0x29 - ADD HL, HL - Add register HL to register HL
            // 0x39 - ADD HL, SP - Add register SP to register HL
            0x09 | 0x19 | 0x29 | 0x39 => match op {
                0x09 => self.alu_add16(Reg16::BC),
                0x19 => self.alu_add16(Reg16::DE),
                0x29 => self.alu_add16(Reg16::HL),
                0x39 => self.alu_add16(Reg16::SP),
                _ => {}
            },

            // 0xE8 - ADD SP, r8 - Add 8-bit signed immediate value to SP
            // Flags: 0 0 H C
            0xE8 => {
                let val = self.imm8() as i8 as i16;
                let sp = self.reg.read16(Reg16::SP) as i16;
                let result = sp.wrapping_add(val);

                self.reg.set_zf(false);
                self.reg.set_nf(false);
                self.reg.set_hf(((sp & 0xF) + (val & 0xF)) > 0xF);
                self.reg.set_cf(((sp & 0xFF) + (val & 0xFF)) > 0xFF);

                self.reg.write16(Reg16::SP, result as u16);
            }

            // DEC r16
            // 0x0B - DEC BC - Decrement register BC
            // 0x1B - DEC DE - Decrement register DE
            // 0x2B - DEC HL - Decrement register HL
            // 0x3B - DEC SP - Decrement register SP
            0x0B | 0x1B | 0x2B | 0x3B => match op {
                0x0B => self
                    .reg
                    .write16(Reg16::BC, self.reg.read16(Reg16::BC).wrapping_sub(1)),
                0x1B => self
                    .reg
                    .write16(Reg16::DE, self.reg.read16(Reg16::DE).wrapping_sub(1)),
                0x2B => self
                    .reg
                    .write16(Reg16::HL, self.reg.read16(Reg16::HL).wrapping_sub(1)),
                0x3B => self
                    .reg
                    .write16(Reg16::SP, self.reg.read16(Reg16::SP).wrapping_sub(1)),
                _ => {}
            },

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

            // 0x08 - LD (a16), SP - Load SP into memory at address a16
            0x08 => {
                let addr = self.imm16();
                self.ld16(addr, self.reg.read16(Reg16::SP));
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

            // LD E, r8
            // 0x58 - LD E, B - Load register B into register E
            // 0x59 - LD E, C - Load register C into register E
            // 0x5A - LD E, D - Load register D into register E
            // 0x5B - LD E, E - Load register E into register E
            // 0x5C - LD E, H - Load register H into register E
            // 0x5D - LD E, L - Load register L into register E
            // 0x5F - LD E, A - Load register A into register E
            0x58 | 0x59 | 0x5A | 0x5B | 0x5C | 0x5D | 0x5F => {
                let val = match op {
                    0x58 => self.reg.read8(Reg8::B),
                    0x59 => self.reg.read8(Reg8::C),
                    0x5A => self.reg.read8(Reg8::D),
                    0x5B => self.reg.read8(Reg8::E),
                    0x5C => self.reg.read8(Reg8::H),
                    0x5D => self.reg.read8(Reg8::L),
                    0x5F => self.reg.read8(Reg8::A),
                    _ => 0x00,
                };
                self.ldr8(Reg8::E, val);
            }

            // LD H, r8
            // 0x60 - LD H, B - Load register B into register H
            // 0x61 - LD H, C - Load register C into register H
            // 0x62 - LD H, D - Load register D into register H
            // 0x63 - LD H, E - Load register E into register H
            // 0x64 - LD H, H - Load register H into register H
            // 0x65 - LD H, L - Load register L into register H
            // 0x67 - LD H, A - Load register A into register H
            0x60 | 0x61 | 0x62 | 0x63 | 0x64 | 0x65 | 0x67 => {
                let val = match op {
                    0x60 => self.reg.read8(Reg8::B),
                    0x61 => self.reg.read8(Reg8::C),
                    0x62 => self.reg.read8(Reg8::D),
                    0x63 => self.reg.read8(Reg8::E),
                    0x64 => self.reg.read8(Reg8::H),
                    0x65 => self.reg.read8(Reg8::L),
                    0x67 => self.reg.read8(Reg8::A),
                    _ => 0x00,
                };
                self.ldr8(Reg8::H, val);
            }

            // LD L, r8
            // 0x68 - LD L, B - Load register B into register L
            // 0x69 - LD L, C - Load register C into register L
            // 0x6A - LD L, D - Load register D into register L
            // 0x6B - LD L, E - Load register E into register L
            // 0x6C - LD L, H - Load register H into register L
            // 0x6D - LD L, L - Load register L into register L
            // 0x6F - LD L, A - Load register A into register L
            0x68 | 0x69 | 0x6A | 0x6B | 0x6C | 0x6D | 0x6F => {
                let val = match op {
                    0x68 => self.reg.read8(Reg8::B),
                    0x69 => self.reg.read8(Reg8::C),
                    0x6A => self.reg.read8(Reg8::D),
                    0x6B => self.reg.read8(Reg8::E),
                    0x6C => self.reg.read8(Reg8::H),
                    0x6D => self.reg.read8(Reg8::L),
                    0x6F => self.reg.read8(Reg8::A),
                    _ => 0x00,
                };
                self.ldr8(Reg8::L, val);
            }

            // LD A, r8
            // 0x78 - LD A, B - Load register B into register A
            // 0x79 - LD A, C - Load register C into register A
            // 0x7A - LD A, D - Load register D into register A
            // 0x7B - LD A, E - Load register E into register A
            // 0x7C - LD A, H - Load register H into register A
            // 0x7D - LD A, L - Load register L into register A
            // 0x7F - LD A, A - Load register A into register A
            0x78 | 0x79 | 0x7A | 0x7B | 0x7C | 0x7D | 0x7F => {
                let val = match op {
                    0x78 => self.reg.read8(Reg8::B),
                    0x79 => self.reg.read8(Reg8::C),
                    0x7A => self.reg.read8(Reg8::D),
                    0x7B => self.reg.read8(Reg8::E),
                    0x7C => self.reg.read8(Reg8::H),
                    0x7D => self.reg.read8(Reg8::L),
                    0x7F => self.reg.read8(Reg8::A),
                    _ => 0x00,
                };
                self.ldr8(Reg8::A, val);
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

            // LD (HL), r8
            // 0x70 - LD (HL), B - Load register B into memory at address HL
            // 0x71 - LD (HL), C - Load register C into memory at address HL
            // 0x72 - LD (HL), D - Load register D into memory at address HL
            // 0x73 - LD (HL), E - Load register E into memory at address HL
            // 0x74 - LD (HL), H - Load register H into memory at address HL
            // 0x75 - LD (HL), L - Load register L into memory at address HL
            // 0x77 - LD (HL), A - Load register A into memory at address HL
            0x70 | 0x71 | 0x72 | 0x73 | 0x74 | 0x75 | 0x77 => {
                let val = match op {
                    0x70 => self.reg.read8(Reg8::B),
                    0x71 => self.reg.read8(Reg8::C),
                    0x72 => self.reg.read8(Reg8::D),
                    0x73 => self.reg.read8(Reg8::E),
                    0x74 => self.reg.read8(Reg8::H),
                    0x75 => self.reg.read8(Reg8::L),
                    0x77 => self.reg.read8(Reg8::A),
                    _ => 0x00,
                };
                self.ld8(self.reg.read16(Reg16::HL), val);
            }

            // POP r16
            // 0xC1 - POP BC - Pop 16-bit value from stack into register BC
            // 0xD1 - POP DE - Pop 16-bit value from stack into register DE
            // 0xE1 - POP HL - Pop 16-bit value from stack into register HL
            // 0xF1 - POP AF - Pop 16-bit value from stack into register AF
            0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                let val = self.stack_pop();
                match op {
                    0xC1 => self.reg.write16(Reg16::BC, val),
                    0xD1 => self.reg.write16(Reg16::DE, val),
                    0xE1 => self.reg.write16(Reg16::HL, val),
                    0xF1 => self.reg.write16(Reg16::AF, val),
                    _ => {}
                }
            }

            // PUSH r16
            // 0xC5 - PUSH BC - Push register BC onto stack
            // 0xD5 - PUSH DE - Push register DE onto stack
            // 0xE5 - PUSH HL - Push register HL onto stack
            // 0xF5 - PUSH AF - Push register AF onto stack
            0xC5 | 0xD5 | 0xE5 | 0xF5 => {
                let val = match op {
                    0xC5 => self.reg.read16(Reg16::BC),
                    0xD5 => self.reg.read16(Reg16::DE),
                    0xE5 => self.reg.read16(Reg16::HL),
                    0xF5 => self.reg.read16(Reg16::AF),
                    _ => 0x0000,
                };
                self.stack_push(val);
            }

            // 0xE0 - LDH (a8), A - Load register A into memory at address 0xFF00 + a8
            0xE0 => {
                let addr = 0xFF00 + self.imm8() as u16;
                self.ld8(addr, self.reg.read8(Reg8::A));
            }

            // 0xF0 - LDH A, (a8) - Load memory at address 0xFF00 + a8 into register A
            0xF0 => {
                let addr = 0xFF00 + self.imm8() as u16;
                let val = self.mem.borrow().read8(addr);
                self.ldr8(Reg8::A, val);
            }

            // 0xE2 - LD (C), A - Load register A into memory at address 0xFF00 + C
            0xE2 => {
                let addr = 0xFF00 + self.reg.read8(Reg8::C) as u16;
                self.ld8(addr, self.reg.read8(Reg8::A));
            }

            // 0xF2 - LD A, (C) - Load memory at address 0xFF00 + C into register A
            0xF2 => {
                let addr = 0xFF00 + self.reg.read8(Reg8::C) as u16;
                let val = self.mem.borrow().read8(addr);
                self.ldr8(Reg8::A, val);
            }

            // 0xF8 - LD HL, SP + r8 - Load the sum of SP and the immediate signed byte into register HL
            // Flags: 0 0 H C
            0xF8 => {
                let r8 = self.imm8() as i8 as i16;
                let sp = self.reg.read16(Reg16::SP) as i16;
                let result = sp.wrapping_add(r8);
                self.reg.set_zf(false);
                self.reg.set_nf(false);
                self.reg.set_hf((sp & 0xF) + (r8 & 0xF) > 0xF);
                self.reg.set_cf((sp & 0xFF) + (r8 & 0xFF) > 0xFF);
                self.ldr16(Reg16::HL, result as u16);
            }

            // 0xF9 - LD SP, HL - Load register HL into register SP
            0xF9 => {
                let val = self.reg.read16(Reg16::HL);
                self.ldr16(Reg16::SP, val);
            }

            // 0xEA - LD (a16), A - Load register A into memory at the absolute 16-bit address a16
            0xEA => {
                let addr = self.imm16();
                self.ld8(addr, self.reg.read8(Reg8::A));
            }

            // 0xFA - LD A, (a16) - Load memory at the absolute 16-bit address a16 into register A
            0xFA => {
                let addr = self.imm16();
                let val = self.mem.borrow().read8(addr);
                self.ldr8(Reg8::A, val);
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

    /// Stack push operation.
    /// Push a 16-bit value (val) onto the stack.
    fn stack_push(&mut self, val: u16) {
        let sp = self.reg.read16(Reg16::SP);
        self.ld16(sp - 2, val);
        self.reg.write16(Reg16::SP, sp - 2);
    }

    /// Stack pop operation.
    /// Pop a 16-bit value from the stack.
    fn stack_pop(&mut self) -> u16 {
        let sp = self.reg.read16(Reg16::SP);
        let val = self.mem.borrow().read16(sp);
        self.reg.write16(Reg16::SP, sp + 2);
        val
    }

    /// ALU 16-bit add operation.
    /// Add a 16-bit value from a 16-bit register to a 16-bit register HL.
    /// Flags: - 0 H C
    fn alu_add16(&mut self, reg: Reg16) {
        let hl = self.reg.read16(Reg16::HL);
        let val = self.reg.read16(reg);
        let result = hl.wrapping_add(val);
        self.reg.set_nf(false);
        self.reg.set_hf((hl & 0x0FFF) + (val & 0x0FFF) > 0x0FFF);
        self.reg.set_cf(hl > 0xFFFF - val);
        self.ldr16(Reg16::HL, result);
    }
}
