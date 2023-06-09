use super::{
    opcodes,
    registers::{Reg16, Reg8},
    Cpu,
};
use log::{info, warn};
use std::collections::HashMap;

impl Cpu {
    /// Executes a CPU operation, returns the number of cycles
    pub(super) fn op_execute(&mut self, op: u8) -> u32 {
        let opcodes: &HashMap<u8, &'static opcodes::OpCode> = &opcodes::OPCODES_MAP;
        let opcode = opcodes.get(&op).unwrap();

        // Jump instructions often have a different number of cycles depending on whether an action is taken or not.
        let mut is_jmp = false;
        let mut jmp_cycles: u32 = 0;

        // Keep track of CB prefix operations cycles
        let mut is_cb = false;
        let mut cb_cycles: u32 = 0;

        info!("{:#02x} {}", opcode.op, &opcode.mnemonic);

        match op {
            // 0x00 - NOP - No operation
            0x00 => {}

            // 0x10 - STOP
            0x10 => {}

            // 0x76 - HALT
            0x76 => {
                self.halt = true;
            }

            // 0xF3 - DI - Disable interrupts
            // NOTE: The IME should be changed not immediately, but after this instruction executes.
            0xF3 => {
                self.ime = false;
            }

            // 0xFB - EI - Enable interrupts
            // NOTE: The IME should be changed not immediately, but after this instruction executes.
            0xFB => {
                self.ime = true;
            }

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
                let addr = 0xFF00 | (self.imm8() as u16);
                self.ld8(addr, self.reg.read8(Reg8::A));

                // Gameboy Boot ROM will write to 0xFF50 to disable itself
                if addr == 0xFF50 {
                    self.boot_rom_enabled = false;
                    println!("Boot ROM disabled");
                }
            }

            // 0xF0 - LDH A, (a8) - Load memory at address 0xFF00 + a8 into register A
            0xF0 => {
                let addr = 0xFF00 | (self.imm8() as u16);
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

            // INC r8
            // 0x04 - INC B - Increment register B
            // 0x0C - INC C - Increment register C
            // 0x14 - INC D - Increment register D
            // 0x1C - INC E - Increment register E
            // 0x24 - INC H - Increment register H
            // 0x2C - INC L - Increment register L
            // 0x3C - INC A - Increment register A
            0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x3C => match op {
                0x04 => self.alu_inc8(Reg8::B),
                0x0C => self.alu_inc8(Reg8::C),
                0x14 => self.alu_inc8(Reg8::D),
                0x1C => self.alu_inc8(Reg8::E),
                0x24 => self.alu_inc8(Reg8::H),
                0x2C => self.alu_inc8(Reg8::L),
                0x3C => self.alu_inc8(Reg8::A),
                _ => {}
            },

            // 0x34 - INC (HL) - Increment memory at register HL
            0x34 => {
                let addr = self.reg.read16(Reg16::HL);
                let val = self.mem.borrow().read8(addr);
                let result = val.wrapping_add(1);
                self.reg.set_zf(result == 0);
                self.reg.set_nf(false);
                self.reg.set_hf((val & 0xF) + 1 > 0xF);
                self.mem.borrow_mut().write8(addr, result);
            }

            // DEC r8
            // 0x05 - DEC B - Decrement register B
            // 0x0D - DEC C - Decrement register C
            // 0x15 - DEC D - Decrement register D
            // 0x1D - DEC E - Decrement register E
            // 0x25 - DEC H - Decrement register H
            // 0x2D - DEC L - Decrement register L
            // 0x3D - DEC A - Decrement register A
            0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x3D => match op {
                0x05 => self.alu_dec8(Reg8::B),
                0x0D => self.alu_dec8(Reg8::C),
                0x15 => self.alu_dec8(Reg8::D),
                0x1D => self.alu_dec8(Reg8::E),
                0x25 => self.alu_dec8(Reg8::H),
                0x2D => self.alu_dec8(Reg8::L),
                0x3D => self.alu_dec8(Reg8::A),
                _ => {}
            },

            // 0x35 - DEC (HL) - Decrement memory at register HL
            0x35 => {
                let addr = self.reg.read16(Reg16::HL);
                let val = self.mem.borrow().read8(addr);
                let result = val.wrapping_sub(1);
                self.reg.set_zf(result == 0);
                self.reg.set_nf(true);
                self.reg.set_hf((val & 0xF) < 1);
                self.mem.borrow_mut().write8(addr, result);
            }

            // 0x27 - DAA - Decimal adjust register A
            0x27 => self.alu_daa(),

            // 0x2F - CPL - Complement register A
            0x2F => self.alu_cpl(),

            // 0x37 - SCF - Set carry flag
            0x37 => self.alu_scf(),

            // 0x3F - CCF - Complement carry flag
            0x3F => self.alu_ccf(),

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

            // ADD A, r8 & ADD A, (HL) & ADD A, d8
            // 0x80 - ADD A, B - Add register B to register A
            // 0x81 - ADD A, C - Add register C to register A
            // 0x82 - ADD A, D - Add register D to register A
            // 0x83 - ADD A, E - Add register E to register A
            // 0x84 - ADD A, H - Add register H to register A
            // 0x85 - ADD A, L - Add register L to register A
            // 0x86 - ADD A, (HL) - Add memory at register HL to register A
            // 0x87 - ADD A, A - Add register A to register A
            // 0xC6 - ADD A, d8 - Add 8-bit immediate value to register A
            0x80 | 0x81 | 0x82 | 0x83 | 0x84 | 0x85 | 0x86 | 0x87 | 0xC6 => match op {
                0x80 => self.alu_addr8(Reg8::B),
                0x81 => self.alu_addr8(Reg8::C),
                0x82 => self.alu_addr8(Reg8::D),
                0x83 => self.alu_addr8(Reg8::E),
                0x84 => self.alu_addr8(Reg8::H),
                0x85 => self.alu_addr8(Reg8::L),
                0x86 => {
                    let val = self.mem.borrow().read8(self.reg.read16(Reg16::HL));
                    self.alu_add8(val);
                }
                0x87 => self.alu_addr8(Reg8::A),
                0xC6 => {
                    let val = self.imm8();
                    self.alu_add8(val);
                }
                _ => {}
            },

            // ADC A, r8 & ADC A, (HL) & ADC A, d8
            // 0x88 - ADC A, B - Add register B + carry flag to register A
            // 0x89 - ADC A, C - Add register C + carry flag to register A
            // 0x8A - ADC A, D - Add register D + carry flag to register A
            // 0x8B - ADC A, E - Add register E + carry flag to register A
            // 0x8C - ADC A, H - Add register H + carry flag to register A
            // 0x8D - ADC A, L - Add register L + carry flag to register A
            // 0x8E - ADC A, (HL) - Add memory at register HL + carry flag to register A
            // 0x8F - ADC A, A - Add register A + carry flag to register A
            // 0xCE - ADC A, d8 - Add 8-bit immediate value + carry flag to register A
            0x88 | 0x89 | 0x8A | 0x8B | 0x8C | 0x8D | 0x8E | 0x8F | 0xCE => match op {
                0x88 => self.alu_adcr8(Reg8::B),
                0x89 => self.alu_adcr8(Reg8::C),
                0x8A => self.alu_adcr8(Reg8::D),
                0x8B => self.alu_adcr8(Reg8::E),
                0x8C => self.alu_adcr8(Reg8::H),
                0x8D => self.alu_adcr8(Reg8::L),
                0x8E => {
                    let val = self.mem.borrow().read8(self.reg.read16(Reg16::HL));
                    self.alu_adc8(val);
                }
                0x8F => self.alu_adcr8(Reg8::A),
                0xCE => {
                    let val = self.imm8();
                    self.alu_adc8(val);
                }
                _ => {}
            },

            // SUB A, r8 & SUB A, (HL) & SUB A, d8
            // 0x90 - SUB A, B - Subtract register B from register A
            // 0x91 - SUB A, C - Subtract register C from register A
            // 0x92 - SUB A, D - Subtract register D from register A
            // 0x93 - SUB A, E - Subtract register E from register A
            // 0x94 - SUB A, H - Subtract register H from register A
            // 0x95 - SUB A, L - Subtract register L from register A
            // 0x96 - SUB A, (HL) - Subtract memory at register HL from register A
            // 0x97 - SUB A, A - Subtract register A from register A
            // 0xD6 - SUB A, d8 - Subtract 8-bit immediate value from register A
            0x90 | 0x91 | 0x92 | 0x93 | 0x94 | 0x95 | 0x96 | 0x97 | 0xD6 => match op {
                0x90 => self.alu_subr8(Reg8::B),
                0x91 => self.alu_subr8(Reg8::C),
                0x92 => self.alu_subr8(Reg8::D),
                0x93 => self.alu_subr8(Reg8::E),
                0x94 => self.alu_subr8(Reg8::H),
                0x95 => self.alu_subr8(Reg8::L),
                0x96 => {
                    let val = self.mem.borrow().read8(self.reg.read16(Reg16::HL));
                    self.alu_sub8(val);
                }
                0x97 => self.alu_subr8(Reg8::A),
                0xD6 => {
                    let val = self.imm8();
                    self.alu_sub8(val);
                }
                _ => {}
            },

            // SBC A, r8 & SBC A, (HL) & SBC A, d8
            // 0x98 - SBC A, B - Subtract register B + carry flag from register A
            // 0x99 - SBC A, C - Subtract register C + carry flag from register A
            // 0x9A - SBC A, D - Subtract register D + carry flag from register A
            // 0x9B - SBC A, E - Subtract register E + carry flag from register A
            // 0x9C - SBC A, H - Subtract register H + carry flag from register A
            // 0x9D - SBC A, L - Subtract register L + carry flag from register A
            // 0x9E - SBC A, (HL) - Subtract memory at register HL + carry flag from register A
            // 0x9F - SBC A, A - Subtract register A + carry flag from register A
            // 0xDE - SBC A, d8 - Subtract 8-bit immediate value + carry flag from register A
            0x98 | 0x99 | 0x9A | 0x9B | 0x9C | 0x9D | 0x9E | 0x9F | 0xDE => match op {
                0x98 => self.alu_sbcr8(Reg8::B),
                0x99 => self.alu_sbcr8(Reg8::C),
                0x9A => self.alu_sbcr8(Reg8::D),
                0x9B => self.alu_sbcr8(Reg8::E),
                0x9C => self.alu_sbcr8(Reg8::H),
                0x9D => self.alu_sbcr8(Reg8::L),
                0x9E => {
                    let val = self.mem.borrow().read8(self.reg.read16(Reg16::HL));
                    self.alu_sbc8(val);
                }
                0x9F => self.alu_sbcr8(Reg8::A),
                0xDE => {
                    let val = self.imm8();
                    self.alu_sbc8(val);
                }
                _ => {}
            },

            // AND A, r8 & AND A, (HL) & AND A, d8
            // 0xA0 - AND A, B - AND register B with register A
            // 0xA1 - AND A, C - AND register C with register A
            // 0xA2 - AND A, D - AND register D with register A
            // 0xA3 - AND A, E - AND register E with register A
            // 0xA4 - AND A, H - AND register H with register A
            // 0xA5 - AND A, L - AND register L with register A
            // 0xA6 - AND A, (HL) - AND memory at register HL with register A
            // 0xA7 - AND A, A - AND register A with register A
            // 0xE6 - AND A, d8 - AND 8-bit immediate value with register A
            0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5 | 0xA6 | 0xA7 | 0xE6 => match op {
                0xA0 => self.alu_andr8(Reg8::B),
                0xA1 => self.alu_andr8(Reg8::C),
                0xA2 => self.alu_andr8(Reg8::D),
                0xA3 => self.alu_andr8(Reg8::E),
                0xA4 => self.alu_andr8(Reg8::H),
                0xA5 => self.alu_andr8(Reg8::L),
                0xA6 => {
                    let val = self.mem.borrow().read8(self.reg.read16(Reg16::HL));
                    self.alu_and8(val);
                }
                0xA7 => self.alu_andr8(Reg8::A),
                0xE6 => {
                    let val = self.imm8();
                    self.alu_and8(val);
                }
                _ => {}
            },

            // XOR A, r8 & XOR A, (HL) & XOR A, d8
            // 0xA8 - XOR A, B - XOR register B with register A
            // 0xA9 - XOR A, C - XOR register C with register A
            // 0xAA - XOR A, D - XOR register D with register A
            // 0xAB - XOR A, E - XOR register E with register A
            // 0xAC - XOR A, H - XOR register H with register A
            // 0xAD - XOR A, L - XOR register L with register A
            // 0xAE - XOR A, (HL) - XOR memory at register HL with register A
            // 0xAF - XOR A, A - XOR register A with register A
            // 0xEE - XOR A, d8 - XOR 8-bit immediate value with register A
            0xA8 | 0xA9 | 0xAA | 0xAB | 0xAC | 0xAD | 0xAE | 0xAF | 0xEE => match op {
                0xA8 => self.alu_xorr8(Reg8::B),
                0xA9 => self.alu_xorr8(Reg8::C),
                0xAA => self.alu_xorr8(Reg8::D),
                0xAB => self.alu_xorr8(Reg8::E),
                0xAC => self.alu_xorr8(Reg8::H),
                0xAD => self.alu_xorr8(Reg8::L),
                0xAE => {
                    let val = self.mem.borrow().read8(self.reg.read16(Reg16::HL));
                    self.alu_xor8(val);
                }
                0xAF => self.alu_xorr8(Reg8::A),
                0xEE => {
                    let val = self.imm8();
                    self.alu_xor8(val);
                }
                _ => {}
            },

            // OR A, r8 & OR A, (HL) & OR A, d8
            // 0xB0 - OR A, B - OR register B with register A
            // 0xB1 - OR A, C - OR register C with register A
            // 0xB2 - OR A, D - OR register D with register A
            // 0xB3 - OR A, E - OR register E with register A
            // 0xB4 - OR A, H - OR register H with register A
            // 0xB5 - OR A, L - OR register L with register A
            // 0xB6 - OR A, (HL) - OR memory at register HL with register A
            // 0xB7 - OR A, A - OR register A with register A
            // 0xF6 - OR A, d8 - OR 8-bit immediate value with register A
            0xB0 | 0xB1 | 0xB2 | 0xB3 | 0xB4 | 0xB5 | 0xB6 | 0xB7 | 0xF6 => match op {
                0xB0 => self.alu_orr8(Reg8::B),
                0xB1 => self.alu_orr8(Reg8::C),
                0xB2 => self.alu_orr8(Reg8::D),
                0xB3 => self.alu_orr8(Reg8::E),
                0xB4 => self.alu_orr8(Reg8::H),
                0xB5 => self.alu_orr8(Reg8::L),
                0xB6 => {
                    let val = self.mem.borrow().read8(self.reg.read16(Reg16::HL));
                    self.alu_or8(val);
                }
                0xB7 => self.alu_orr8(Reg8::A),
                0xF6 => {
                    let val = self.imm8();
                    self.alu_or8(val);
                }
                _ => {}
            },

            // CP A, r8 & CP A, (HL) & CP A, d8
            // 0xB8 - CP A, B - Compare register B with register A
            // 0xB9 - CP A, C - Compare register C with register A
            // 0xBA - CP A, D - Compare register D with register A
            // 0xBB - CP A, E - Compare register E with register A
            // 0xBC - CP A, H - Compare register H with register A
            // 0xBD - CP A, L - Compare register L with register A
            // 0xBE - CP A, (HL) - Compare memory at register HL with register A
            // 0xBF - CP A, A - Compare register A with register A
            // 0xFE - CP A, d8 - Compare 8-bit immediate value with register A
            0xB8 | 0xB9 | 0xBA | 0xBB | 0xBC | 0xBD | 0xBE | 0xBF | 0xFE => match op {
                0xB8 => self.alu_cpr8(Reg8::B),
                0xB9 => self.alu_cpr8(Reg8::C),
                0xBA => self.alu_cpr8(Reg8::D),
                0xBB => self.alu_cpr8(Reg8::E),
                0xBC => self.alu_cpr8(Reg8::H),
                0xBD => self.alu_cpr8(Reg8::L),
                0xBE => {
                    let val = self.mem.borrow().read8(self.reg.read16(Reg16::HL));
                    self.alu_cp8(val);
                }
                0xBF => self.alu_cpr8(Reg8::A),
                0xFE => {
                    let val = self.imm8();
                    self.alu_cp8(val);
                }
                _ => {}
            },

            // 0xC2 - JP NZ, a16 - Jump to 16-bit immediate value if zero flag is not set
            // Cycles if taken: 16
            // Cycles if not taken: 12
            0xC2 => {
                let addr = self.imm16();
                if !self.reg.zf() {
                    self.reg.write16(Reg16::PC, addr);
                    jmp_cycles = 16;
                } else {
                    jmp_cycles = 12;
                }
                is_jmp = true;
            }

            // 0xC3 - JP a16 - Jump to 16-bit immediate value
            0xC3 => {
                let addr = self.imm16();
                self.reg.write16(Reg16::PC, addr);
                jmp_cycles = opcode.cycles;
                is_jmp = true;
            }

            // 0xCA - JP Z, a16 - Jump to 16-bit immediate value if zero flag is set
            // Cycles if taken: 16
            // Cycles if not taken: 12
            0xCA => {
                let addr = self.imm16();
                if self.reg.zf() {
                    self.reg.write16(Reg16::PC, addr);
                    jmp_cycles = 16;
                } else {
                    jmp_cycles = 12;
                }
                is_jmp = true;
            }

            // 0xD2 - JP NC, a16 - Jump to 16-bit immediate value if carry flag is not set
            // Cycles if taken: 16
            // Cycles if not taken: 12
            0xD2 => {
                let addr = self.imm16();
                if !self.reg.cf() {
                    self.reg.write16(Reg16::PC, addr);
                    jmp_cycles = 16;
                } else {
                    jmp_cycles = 12;
                }
                is_jmp = true;
            }

            // 0xDA - JP C, a16 - Jump to 16-bit immediate value if carry flag is set
            // Cycles if taken: 16
            // Cycles if not taken: 12
            0xDA => {
                let addr = self.imm16();
                if self.reg.cf() {
                    self.reg.write16(Reg16::PC, addr);
                    jmp_cycles = 16;
                } else {
                    jmp_cycles = 12;
                }
                is_jmp = true;
            }

            // 0xE9 - JP (HL) - Jump to address stored in HL
            0xE9 => {
                let addr = self.reg.read16(Reg16::HL);
                self.reg.write16(Reg16::PC, addr);
                jmp_cycles = opcode.cycles;
                is_jmp = true;
            }

            // 0x18 - JR r8 - Add 8-bit signed immediate value to PC
            0x18 => {
                let val = self.imm8() as i8;
                let addr =
                    ((u32::from(self.reg.read16(Reg16::PC)) as i32) + (i32::from(val))) as u16;
                self.reg.write16(Reg16::PC, addr);
                jmp_cycles = opcode.cycles;
                is_jmp = true;
            }

            // 0x20 - JR NZ, r8 - Add 8-bit signed immediate value to PC if zero flag is not set
            // Cycles if taken: 12
            // Cycles if not taken: 8
            0x20 => {
                let val = self.imm8() as i8;
                let addr =
                    ((u32::from(self.reg.read16(Reg16::PC)) as i32) + (i32::from(val))) as u16;
                if !self.reg.zf() {
                    self.reg.write16(Reg16::PC, addr);
                    jmp_cycles = 12;
                } else {
                    jmp_cycles = 8;
                }
                is_jmp = true;
            }

            // 0x28 - JR Z, r8 - Add 8-bit signed immediate value to PC if zero flag is set
            // Cycles if taken: 12
            // Cycles if not taken: 8
            0x28 => {
                let val = self.imm8() as i8;
                let addr =
                    ((u32::from(self.reg.read16(Reg16::PC)) as i32) + (i32::from(val))) as u16;
                if self.reg.zf() {
                    self.reg.write16(Reg16::PC, addr);
                    jmp_cycles = 12;
                } else {
                    jmp_cycles = 8;
                }
                is_jmp = true;
            }

            // 0x30 - JR NC, r8 - Add 8-bit signed immediate value to PC if carry flag is not set
            // Cycles if taken: 12
            // Cycles if not taken: 8
            0x30 => {
                let val = self.imm8() as i8;
                let addr =
                    ((u32::from(self.reg.read16(Reg16::PC)) as i32) + (i32::from(val))) as u16;
                if !self.reg.cf() {
                    self.reg.write16(Reg16::PC, addr);
                    jmp_cycles = 12;
                } else {
                    jmp_cycles = 8;
                }
                is_jmp = true;
            }

            // 0x38 - JR C, r8 - Add 8-bit signed immediate value to PC if carry flag is set
            // Cycles if taken: 12
            // Cycles if not taken: 8
            0x38 => {
                let val = self.imm8() as i8;
                let addr =
                    ((u32::from(self.reg.read16(Reg16::PC)) as i32) + (i32::from(val))) as u16;
                if self.reg.cf() {
                    self.reg.write16(Reg16::PC, addr);
                    jmp_cycles = 12;
                } else {
                    jmp_cycles = 8;
                }
                is_jmp = true;
            }

            // 0xC4 - CALL NZ, a16 - Call 16-bit immediate value if zero flag is not set
            // Cycles if taken: 24
            // Cycles if not taken: 12
            0xC4 => {
                let addr = self.imm16();
                let pc = self.reg.read16(Reg16::PC);
                if !self.reg.zf() {
                    self.stack_push(pc);
                    self.reg.write16(Reg16::PC, addr);
                    jmp_cycles = 24;
                } else {
                    jmp_cycles = 12;
                }
                is_jmp = true;
            }

            // 0xCC - CALL Z, a16 - Call 16-bit immediate value if zero flag is set
            // Cycles if taken: 24
            // Cycles if not taken: 12
            0xCC => {
                let addr = self.imm16();
                let pc = self.reg.read16(Reg16::PC);
                if self.reg.zf() {
                    self.stack_push(pc);
                    self.reg.write16(Reg16::PC, addr);
                    jmp_cycles = 24;
                } else {
                    jmp_cycles = 12;
                }
                is_jmp = true;
            }

            // 0xCD - CALL a16 - Call 16-bit immediate value
            0xCD => {
                let addr = self.imm16();
                let pc = self.reg.read16(Reg16::PC);
                self.stack_push(pc);
                self.reg.write16(Reg16::PC, addr);
                jmp_cycles = opcode.cycles;
                is_jmp = true;
            }

            // 0xD4 - CALL NC, a16 - Call 16-bit immediate value if carry flag is not set
            // Cycles if taken: 24
            // Cycles if not taken: 12
            0xD4 => {
                let addr = self.imm16();
                let pc = self.reg.read16(Reg16::PC);
                if !self.reg.cf() {
                    self.stack_push(pc);
                    self.reg.write16(Reg16::PC, addr);
                    jmp_cycles = 24;
                } else {
                    jmp_cycles = 12;
                }
                is_jmp = true;
            }

            // 0xDC - CALL C, a16 - Call 16-bit immediate value if carry flag is set
            // Cycles if taken: 24
            // Cycles if not taken: 12
            0xDC => {
                let addr = self.imm16();
                let pc = self.reg.read16(Reg16::PC);
                if self.reg.cf() {
                    self.stack_push(pc);
                    self.reg.write16(Reg16::PC, addr);
                    jmp_cycles = 24;
                } else {
                    jmp_cycles = 12;
                }
                is_jmp = true;
            }

            // 0xC0 - RET NZ - Return if zero flag is not set
            // Cycles if taken: 20
            // Cycles if not taken: 8
            0xC0 => {
                if !self.reg.zf() {
                    let addr = self.stack_pop();
                    self.reg.write16(Reg16::PC, addr);
                    jmp_cycles = 20;
                } else {
                    jmp_cycles = 8;
                }
                is_jmp = true;
            }

            // 0xC8 - RET Z - Return if zero flag is set
            // Cycles if taken: 20
            // Cycles if not taken: 8
            0xC8 => {
                if self.reg.zf() {
                    let addr = self.stack_pop();
                    self.reg.write16(Reg16::PC, addr);
                    jmp_cycles = 20;
                } else {
                    jmp_cycles = 8;
                }
                is_jmp = true;
            }

            // 0xC9 - RET - Return
            0xC9 => {
                let addr = self.stack_pop();
                self.reg.write16(Reg16::PC, addr);
                jmp_cycles = opcode.cycles;
                is_jmp = true;
            }

            // 0xD0 - RET NC - Return if carry flag is not set
            // Cycles if taken: 20
            // Cycles if not taken: 8
            0xD0 => {
                if !self.reg.cf() {
                    let addr = self.stack_pop();
                    self.reg.write16(Reg16::PC, addr);
                    jmp_cycles = 20;
                } else {
                    jmp_cycles = 8;
                }
                is_jmp = true;
            }

            // 0xD8 - RET C - Return if carry flag is set
            // Cycles if taken: 20
            // Cycles if not taken: 8
            0xD8 => {
                if self.reg.cf() {
                    let addr = self.stack_pop();
                    self.reg.write16(Reg16::PC, addr);
                    jmp_cycles = 20;
                } else {
                    jmp_cycles = 8;
                }
                is_jmp = true;
            }

            // 0xD9 - RETI - Return and enable interrupts
            0xD9 => {
                let addr = self.stack_pop();
                self.reg.write16(Reg16::PC, addr);
                self.ime = true;
                jmp_cycles = opcode.cycles;
                is_jmp = true;
            }

            // 0xC7 - RST 00H - Restart at address 0x0000
            // 0xCF - RST 08H - Restart at address 0x0008
            // 0xD7 - RST 10H - Restart at address 0x0010
            // 0xDF - RST 18H - Restart at address 0x0018
            // 0xE7 - RST 20H - Restart at address 0x0020
            // 0xEF - RST 28H - Restart at address 0x0028
            // 0xF7 - RST 30H - Restart at address 0x0030
            // 0xFF - RST 38H - Restart at address 0x0038
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
                let addr = (op & 0x38) as u16;
                self.stack_push(self.reg.read16(Reg16::PC));
                self.reg.write16(Reg16::PC, addr);
                jmp_cycles = opcode.cycles;
                is_jmp = true;
            }

            // 0x07 - RLCA - Rotate A left. Old bit 7 to Carry flag.
            0x07 => {
                let val = self.reg.read8(Reg8::A);
                let result = self.alu_rlc(val);
                self.reg.write8(Reg8::A, result);
                // Flag Z is reset here.
                self.reg.set_zf(false);
            }

            // 0x0F - RRCA - Rotate A right. Old bit 0 to Carry flag.
            0x0F => {
                let val = self.reg.read8(Reg8::A);
                let result = self.alu_rrc(val);
                self.reg.write8(Reg8::A, result);
                // Flag Z is reset here.
                self.reg.set_zf(false);
            }

            // 0x17 - RLA - Rotate A left through Carry flag.
            0x17 => {
                let val = self.reg.read8(Reg8::A);
                let result = self.alu_rl(val);
                self.reg.write8(Reg8::A, result);
                // Flag Z is reset here.
                self.reg.set_zf(false);
            }

            // 0x1F - RRA - Rotate A right through Carry flag.
            0x1F => {
                let val = self.reg.read8(Reg8::A);
                let result = self.alu_rr(val);
                self.reg.write8(Reg8::A, result);
                // Flag Z is reset here.
                self.reg.set_zf(false);
            }

            // 0xCB - CB prefix
            0xCB => {
                let cb_op = self.imm8();
                cb_cycles = self.cb_op_execute(cb_op);
                is_cb = true;
            }

            _ => {
                warn!("Illegal opcode: {:#02x}.", op);
            }
        }

        if is_jmp {
            return jmp_cycles;
        }
        if is_cb {
            return cb_cycles;
        }
        opcode.cycles
    }

    /// Executes a CB-prefix operation, returns the number of cycles
    fn cb_op_execute(&mut self, op: u8) -> u32 {
        let cb_opcodes: &HashMap<u8, &'static opcodes::OpCode> = &opcodes::CB_OPCODES_MAP;
        let cb_opcode = cb_opcodes.get(&op).unwrap();

        info!("CB {:#02x} {}", cb_opcode.op, &cb_opcode.mnemonic);

        match op {
            // RLC r8
            // 0x00 - RLC B
            // 0x01 - RLC C
            // 0x02 - RLC D
            // 0x03 - RLC E
            // 0x04 - RLC H
            // 0x05 - RLC L
            // 0x07 - RLC A
            0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x07 => {
                let (reg, result) = match op {
                    0x00 => (Reg8::B, self.alu_rlc(self.reg.read8(Reg8::B))),
                    0x01 => (Reg8::C, self.alu_rlc(self.reg.read8(Reg8::C))),
                    0x02 => (Reg8::D, self.alu_rlc(self.reg.read8(Reg8::D))),
                    0x03 => (Reg8::E, self.alu_rlc(self.reg.read8(Reg8::E))),
                    0x04 => (Reg8::H, self.alu_rlc(self.reg.read8(Reg8::H))),
                    0x05 => (Reg8::L, self.alu_rlc(self.reg.read8(Reg8::L))),
                    0x07 => (Reg8::A, self.alu_rlc(self.reg.read8(Reg8::A))),
                    _ => unreachable!(),
                };
                self.reg.write8(reg, result);
            }

            // 0x06 - RLC (HL)
            0x06 => {
                let hl = self.reg.read16(Reg16::HL);
                let val = self.mem.borrow().read8(hl);
                let result = self.alu_rlc(val);
                self.mem.borrow_mut().write8(hl, result);
            }

            // RRC r8
            // 0x08 - RRC B
            // 0x09 - RRC C
            // 0x0A - RRC D
            // 0x0B - RRC E
            // 0x0C - RRC H
            // 0x0D - RRC L
            // 0x0F - RRC A
            0x08 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D | 0x0F => {
                let (reg, result) = match op {
                    0x08 => (Reg8::B, self.alu_rrc(self.reg.read8(Reg8::B))),
                    0x09 => (Reg8::C, self.alu_rrc(self.reg.read8(Reg8::C))),
                    0x0A => (Reg8::D, self.alu_rrc(self.reg.read8(Reg8::D))),
                    0x0B => (Reg8::E, self.alu_rrc(self.reg.read8(Reg8::E))),
                    0x0C => (Reg8::H, self.alu_rrc(self.reg.read8(Reg8::H))),
                    0x0D => (Reg8::L, self.alu_rrc(self.reg.read8(Reg8::L))),
                    0x0F => (Reg8::A, self.alu_rrc(self.reg.read8(Reg8::A))),
                    _ => unreachable!(),
                };
                self.reg.write8(reg, result);
            }

            // 0x0E - RRC (HL)
            0x0E => {
                let hl = self.reg.read16(Reg16::HL);
                let val = self.mem.borrow().read8(hl);
                let result = self.alu_rrc(val);
                self.mem.borrow_mut().write8(hl, result);
            }

            // RL r8
            // 0x10 - RL B
            // 0x11 - RL C
            // 0x12 - RL D
            // 0x13 - RL E
            // 0x14 - RL H
            // 0x15 - RL L
            // 0x17 - RL A
            0x10 | 0x11 | 0x12 | 0x13 | 0x14 | 0x15 | 0x17 => {
                let (reg, result) = match op {
                    0x10 => (Reg8::B, self.alu_rl(self.reg.read8(Reg8::B))),
                    0x11 => (Reg8::C, self.alu_rl(self.reg.read8(Reg8::C))),
                    0x12 => (Reg8::D, self.alu_rl(self.reg.read8(Reg8::D))),
                    0x13 => (Reg8::E, self.alu_rl(self.reg.read8(Reg8::E))),
                    0x14 => (Reg8::H, self.alu_rl(self.reg.read8(Reg8::H))),
                    0x15 => (Reg8::L, self.alu_rl(self.reg.read8(Reg8::L))),
                    0x17 => (Reg8::A, self.alu_rl(self.reg.read8(Reg8::A))),
                    _ => unreachable!(),
                };
                self.reg.write8(reg, result);
            }

            // 0x16 - RL (HL)
            0x16 => {
                let hl = self.reg.read16(Reg16::HL);
                let val = self.mem.borrow().read8(hl);
                let result = self.alu_rl(val);
                self.mem.borrow_mut().write8(hl, result);
            }

            // RR r8
            // 0x18 - RR B
            // 0x19 - RR C
            // 0x1A - RR D
            // 0x1B - RR E
            // 0x1C - RR H
            // 0x1D - RR L
            // 0x1F - RR A
            0x18 | 0x19 | 0x1A | 0x1B | 0x1C | 0x1D | 0x1F => {
                let (reg, result) = match op {
                    0x18 => (Reg8::B, self.alu_rr(self.reg.read8(Reg8::B))),
                    0x19 => (Reg8::C, self.alu_rr(self.reg.read8(Reg8::C))),
                    0x1A => (Reg8::D, self.alu_rr(self.reg.read8(Reg8::D))),
                    0x1B => (Reg8::E, self.alu_rr(self.reg.read8(Reg8::E))),
                    0x1C => (Reg8::H, self.alu_rr(self.reg.read8(Reg8::H))),
                    0x1D => (Reg8::L, self.alu_rr(self.reg.read8(Reg8::L))),
                    0x1F => (Reg8::A, self.alu_rr(self.reg.read8(Reg8::A))),
                    _ => unreachable!(),
                };
                self.reg.write8(reg, result);
            }

            // 0x1E - RR (HL)
            0x1E => {
                let hl = self.reg.read16(Reg16::HL);
                let val = self.mem.borrow().read8(hl);
                let result = self.alu_rr(val);
                self.mem.borrow_mut().write8(hl, result);
            }

            // SLA r8
            // 0x20 - SLA B
            // 0x21 - SLA C
            // 0x22 - SLA D
            // 0x23 - SLA E
            // 0x24 - SLA H
            // 0x25 - SLA L
            // 0x27 - SLA A
            0x20 | 0x21 | 0x22 | 0x23 | 0x24 | 0x25 | 0x27 => {
                let (reg, result) = match op {
                    0x20 => (Reg8::B, self.alu_sla(self.reg.read8(Reg8::B))),
                    0x21 => (Reg8::C, self.alu_sla(self.reg.read8(Reg8::C))),
                    0x22 => (Reg8::D, self.alu_sla(self.reg.read8(Reg8::D))),
                    0x23 => (Reg8::E, self.alu_sla(self.reg.read8(Reg8::E))),
                    0x24 => (Reg8::H, self.alu_sla(self.reg.read8(Reg8::H))),
                    0x25 => (Reg8::L, self.alu_sla(self.reg.read8(Reg8::L))),
                    0x27 => (Reg8::A, self.alu_sla(self.reg.read8(Reg8::A))),
                    _ => unreachable!(),
                };
                self.reg.write8(reg, result);
            }

            // 0x26 - SLA (HL)
            0x26 => {
                let hl = self.reg.read16(Reg16::HL);
                let val = self.mem.borrow().read8(hl);
                let result = self.alu_sla(val);
                self.mem.borrow_mut().write8(hl, result);
            }

            // SRA r8
            // 0x28 - SRA B
            // 0x29 - SRA C
            // 0x2A - SRA D
            // 0x2B - SRA E
            // 0x2C - SRA H
            // 0x2D - SRA L
            // 0x2F - SRA A
            0x28 | 0x29 | 0x2A | 0x2B | 0x2C | 0x2D | 0x2F => {
                let (reg, result) = match op {
                    0x28 => (Reg8::B, self.alu_sra(self.reg.read8(Reg8::B))),
                    0x29 => (Reg8::C, self.alu_sra(self.reg.read8(Reg8::C))),
                    0x2A => (Reg8::D, self.alu_sra(self.reg.read8(Reg8::D))),
                    0x2B => (Reg8::E, self.alu_sra(self.reg.read8(Reg8::E))),
                    0x2C => (Reg8::H, self.alu_sra(self.reg.read8(Reg8::H))),
                    0x2D => (Reg8::L, self.alu_sra(self.reg.read8(Reg8::L))),
                    0x2F => (Reg8::A, self.alu_sra(self.reg.read8(Reg8::A))),
                    _ => unreachable!(),
                };
                self.reg.write8(reg, result);
            }

            // 0x2E - SRA (HL)
            0x2E => {
                let hl = self.reg.read16(Reg16::HL);
                let val = self.mem.borrow().read8(hl);
                let result = self.alu_sra(val);
                self.mem.borrow_mut().write8(hl, result);
            }

            // SWAP r8
            // 0x30 - SWAP B
            // 0x31 - SWAP C
            // 0x32 - SWAP D
            // 0x33 - SWAP E
            // 0x34 - SWAP H
            // 0x35 - SWAP L
            // 0x37 - SWAP A
            0x30 | 0x31 | 0x32 | 0x33 | 0x34 | 0x35 | 0x37 => {
                let (reg, result) = match op {
                    0x30 => (Reg8::B, self.alu_swap(self.reg.read8(Reg8::B))),
                    0x31 => (Reg8::C, self.alu_swap(self.reg.read8(Reg8::C))),
                    0x32 => (Reg8::D, self.alu_swap(self.reg.read8(Reg8::D))),
                    0x33 => (Reg8::E, self.alu_swap(self.reg.read8(Reg8::E))),
                    0x34 => (Reg8::H, self.alu_swap(self.reg.read8(Reg8::H))),
                    0x35 => (Reg8::L, self.alu_swap(self.reg.read8(Reg8::L))),
                    0x37 => (Reg8::A, self.alu_swap(self.reg.read8(Reg8::A))),
                    _ => unreachable!(),
                };
                self.reg.write8(reg, result);
            }

            // 0x36 - SWAP (HL)
            0x36 => {
                let hl = self.reg.read16(Reg16::HL);
                let val = self.mem.borrow().read8(hl);
                let result = self.alu_swap(val);
                self.mem.borrow_mut().write8(hl, result);
            }

            // SRL r8
            // 0x38 - SRL B
            // 0x39 - SRL C
            // 0x3A - SRL D
            // 0x3B - SRL E
            // 0x3C - SRL H
            // 0x3D - SRL L
            // 0x3F - SRL A
            0x38 | 0x39 | 0x3A | 0x3B | 0x3C | 0x3D | 0x3F => {
                let (reg, result) = match op {
                    0x38 => (Reg8::B, self.alu_srl(self.reg.read8(Reg8::B))),
                    0x39 => (Reg8::C, self.alu_srl(self.reg.read8(Reg8::C))),
                    0x3A => (Reg8::D, self.alu_srl(self.reg.read8(Reg8::D))),
                    0x3B => (Reg8::E, self.alu_srl(self.reg.read8(Reg8::E))),
                    0x3C => (Reg8::H, self.alu_srl(self.reg.read8(Reg8::H))),
                    0x3D => (Reg8::L, self.alu_srl(self.reg.read8(Reg8::L))),
                    0x3F => (Reg8::A, self.alu_srl(self.reg.read8(Reg8::A))),
                    _ => unreachable!(),
                };
                self.reg.write8(reg, result);
            }

            // 0x3E - SRL (HL)
            0x3E => {
                let hl = self.reg.read16(Reg16::HL);
                let val = self.mem.borrow().read8(hl);
                let result = self.alu_srl(val);
                self.mem.borrow_mut().write8(hl, result);
            }

            // BIT b, r8
            // b = 0 - 7, r8 = B, C, D, E, H, L, (HL), A
            // 0x40 .. 0x47 - BIT 0, r8
            // 0x48 .. 0x4F - BIT 1, r8
            // 0x50 .. 0x57 - BIT 2, r8
            // 0x58 .. 0x5F - BIT 3, r8
            // 0x60 .. 0x67 - BIT 4, r8
            // 0x68 .. 0x6F - BIT 5, r8
            // 0x70 .. 0x77 - BIT 6, r8
            // 0x78 .. 0x7F - BIT 7, r8
            0x40..=0x7F => {
                let bit = (op >> 3) & 0x7;
                let val = match op & 0x7 {
                    0x0 => self.reg.read8(Reg8::B),
                    0x1 => self.reg.read8(Reg8::C),
                    0x2 => self.reg.read8(Reg8::D),
                    0x3 => self.reg.read8(Reg8::E),
                    0x4 => self.reg.read8(Reg8::H),
                    0x5 => self.reg.read8(Reg8::L),
                    0x6 => {
                        let hl = self.reg.read16(Reg16::HL);
                        self.mem.borrow().read8(hl)
                    }
                    0x7 => self.reg.read8(Reg8::A),
                    _ => unreachable!(),
                };
                self.alu_bit(bit, val);
            }

            // RES b, r8
            // b = 0 - 7, r8 = B, C, D, E, H, L, (HL), A
            // 0x80 .. 0x87 - RES 0, r8
            // 0x88 .. 0x8F - RES 1, r8
            // 0x90 .. 0x97 - RES 2, r8
            // 0x98 .. 0x9F - RES 3, r8
            // 0xA0 .. 0xA7 - RES 4, r8
            // 0xA8 .. 0xAF - RES 5, r8
            // 0xB0 .. 0xB7 - RES 6, r8
            // 0xB8 .. 0xBF - RES 7, r8
            0x80..=0xBF => {
                let bit = (op >> 3) & 0x7;
                let (reg, result) = match op & 0x7 {
                    0x0 => (Reg8::B, self.alu_res(bit, self.reg.read8(Reg8::B))),
                    0x1 => (Reg8::C, self.alu_res(bit, self.reg.read8(Reg8::C))),
                    0x2 => (Reg8::D, self.alu_res(bit, self.reg.read8(Reg8::D))),
                    0x3 => (Reg8::E, self.alu_res(bit, self.reg.read8(Reg8::E))),
                    0x4 => (Reg8::H, self.alu_res(bit, self.reg.read8(Reg8::H))),
                    0x5 => (Reg8::L, self.alu_res(bit, self.reg.read8(Reg8::L))),
                    0x6 => {
                        let hl = self.reg.read16(Reg16::HL);
                        let val = self.mem.borrow().read8(hl);
                        let result = self.alu_res(bit, val);
                        self.mem.borrow_mut().write8(hl, result);
                        (Reg8::B, 0)
                    }
                    0x7 => (Reg8::A, self.alu_res(bit, self.reg.read8(Reg8::A))),
                    _ => unreachable!(),
                };
                if op & 0x7 != 0x6 {
                    self.reg.write8(reg, result);
                }
            }

            // SET b, r8
            // b = 0 - 7, r8 = B, C, D, E, H, L, (HL), A
            // 0xC0 .. 0xC7 - SET 0, r8
            // 0xC8 .. 0xCF - SET 1, r8
            // 0xD0 .. 0xD7 - SET 2, r8
            // 0xD8 .. 0xDF - SET 3, r8
            // 0xE0 .. 0xE7 - SET 4, r8
            // 0xE8 .. 0xEF - SET 5, r8
            // 0xF0 .. 0xF7 - SET 6, r8
            // 0xF8 .. 0xFF - SET 7, r8
            0xC0..=0xFF => {
                let bit = (op >> 3) & 0x7;
                let (reg, result) = match op & 0x7 {
                    0x0 => (Reg8::B, self.alu_set(bit, self.reg.read8(Reg8::B))),
                    0x1 => (Reg8::C, self.alu_set(bit, self.reg.read8(Reg8::C))),
                    0x2 => (Reg8::D, self.alu_set(bit, self.reg.read8(Reg8::D))),
                    0x3 => (Reg8::E, self.alu_set(bit, self.reg.read8(Reg8::E))),
                    0x4 => (Reg8::H, self.alu_set(bit, self.reg.read8(Reg8::H))),
                    0x5 => (Reg8::L, self.alu_set(bit, self.reg.read8(Reg8::L))),
                    0x6 => {
                        let hl = self.reg.read16(Reg16::HL);
                        let val = self.mem.borrow().read8(hl);
                        let result = self.alu_set(bit, val);
                        self.mem.borrow_mut().write8(hl, result);
                        (Reg8::B, 0)
                    }
                    0x7 => (Reg8::A, self.alu_set(bit, self.reg.read8(Reg8::A))),
                    _ => unreachable!(),
                };
                if op & 0x7 != 0x6 {
                    self.reg.write8(reg, result);
                }
            }
        }
        cb_opcode.cycles
    }
}

impl Cpu {
    /// Fetch the immediate byte (u8).
    pub(super) fn imm8(&mut self) -> u8 {
        let val = self.mem.borrow().read8(self.reg.read16(Reg16::PC));
        self.reg.inc_pc(1);
        val
    }

    /// Fetch the immediate word (u16).
    fn imm16(&mut self) -> u16 {
        let val = self.mem.borrow().read16(self.reg.read16(Reg16::PC));
        self.reg.inc_pc(2);
        val
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
    pub(super) fn stack_push(&mut self, val: u16) {
        self.reg.dec_sp(2);
        let sp = self.reg.read16(Reg16::SP);
        self.mem.borrow_mut().write16(sp, val);
        //self.ld16(sp - 2, val);
        //self.reg.dec_sp(2);
    }

    /// Stack pop operation.
    /// Pop a 16-bit value from the stack.
    fn stack_pop(&mut self) -> u16 {
        let sp = self.reg.read16(Reg16::SP);
        let val = self.mem.borrow().read16(sp);
        self.reg.inc_sp(2);
        val
    }

    /// ALU 8-bit increment operation.
    /// Increment an 8-bit value from an 8-bit register.
    /// Flags: Z 0 H -
    fn alu_inc8(&mut self, reg: Reg8) {
        let val = self.reg.read8(reg);
        let result = val.wrapping_add(1);
        self.reg.set_zf(result == 0);
        self.reg.set_nf(false);
        self.reg.set_hf((val & 0x0F) + 1 > 0x0F);
        self.reg.write8(reg, result);
    }

    /// ALU 8-bit decrement operation.
    /// Decrement an 8-bit value from an 8-bit register.
    /// Flags: Z 1 H -
    fn alu_dec8(&mut self, reg: Reg8) {
        let val = self.reg.read8(reg);
        let result = val.wrapping_sub(1);
        self.reg.set_zf(result == 0);
        self.reg.set_nf(true);
        self.reg.set_hf((val & 0x0F) == 0);
        self.reg.write8(reg, result);
    }

    /// ALU 8-bit add operation.
    /// Add a 8-bit value from a 8-bit register to a 8-bit register A.
    /// Flags: Z 0 H C
    fn alu_addr8(&mut self, reg: Reg8) {
        let a = self.reg.read8(Reg8::A);
        let val = self.reg.read8(reg);
        let result = a.wrapping_add(val);
        self.reg.set_zf(result == 0);
        self.reg.set_nf(false);
        self.reg.set_hf((a & 0x0F) + (val & 0x0F) > 0x0F);
        self.reg.set_cf(u16::from(a) + u16::from(val) > 0xFF);
        self.reg.write8(Reg8::A, result);
    }

    /// ALU 8-bit add operation.
    /// Add a 8-bit value to a 8-bit register A.
    /// Flags: Z 0 H C
    fn alu_add8(&mut self, val: u8) {
        let a = self.reg.read8(Reg8::A);
        let result = a.wrapping_add(val);
        self.reg.set_zf(result == 0);
        self.reg.set_nf(false);
        self.reg.set_hf((a & 0x0F) + (val & 0x0F) > 0x0F);
        self.reg.set_cf(u16::from(a) + u16::from(val) > 0xFF);
        self.reg.write8(Reg8::A, result);
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
        self.reg.write16(Reg16::HL, result);
    }

    /// ALU 8-bit add carry operation.
    /// Add a 8-bit value from a 8-bit register to a 8-bit register A with carry. (A = A + val + C).
    /// Flags: Z 0 H C
    /// NOTE: This is the same as alu_add8, but with the carry flag added.
    fn alu_adcr8(&mut self, reg: Reg8) {
        let a = self.reg.read8(Reg8::A);
        let val = self.reg.read8(reg);
        let c = if self.reg.cf() { 1 } else { 0 };
        let result = a.wrapping_add(val).wrapping_add(c);
        self.reg.set_zf(result == 0);
        self.reg.set_nf(false);
        self.reg.set_hf((a & 0x0F) + (val & 0x0F) + c > 0x0F);
        self.reg
            .set_cf(u16::from(a) + u16::from(val) + u16::from(c) > 0xFF);
        self.reg.write8(Reg8::A, result);
    }

    /// ALU 8-bit add carry operation.
    /// Add a 8-bit value a 8-bit register A with carry. (A = A + val + C).
    /// Flags: Z 0 H C
    /// NOTE: This is the same as alu_add8, but with the carry flag added.
    fn alu_adc8(&mut self, val: u8) {
        let a = self.reg.read8(Reg8::A);
        let c = if self.reg.cf() { 1 } else { 0 };
        let result = a.wrapping_add(val).wrapping_add(c);
        self.reg.set_zf(result == 0);
        self.reg.set_nf(false);
        self.reg.set_hf((a & 0x0F) + (val & 0x0F) + c > 0x0F);
        self.reg
            .set_cf(u16::from(a) + u16::from(val) + u16::from(c) > 0xFF);
        self.reg.write8(Reg8::A, result);
    }

    /// ALU 8-bit subtract operation.
    /// Subtract a 8-bit value from a 8-bit register from a 8-bit register A.
    /// Flags: Z 1 H C
    fn alu_subr8(&mut self, reg: Reg8) {
        let a = self.reg.read8(Reg8::A);
        let val = self.reg.read8(reg);
        let result = a.wrapping_sub(val);
        self.reg.set_zf(result == 0);
        self.reg.set_nf(true);
        self.reg.set_hf((a & 0x0F) < (val & 0x0F));
        self.reg.set_cf(u16::from(a) < u16::from(val));
        self.reg.write8(Reg8::A, result);
    }

    /// ALU 8-bit subtract operation.
    /// Subtract a 8-bit value from a 8-bit register A.
    /// Flags: Z 1 H C
    fn alu_sub8(&mut self, val: u8) {
        let a = self.reg.read8(Reg8::A);
        let result = a.wrapping_sub(val);
        self.reg.set_zf(result == 0);
        self.reg.set_nf(true);
        self.reg.set_hf((a & 0x0F) < (val & 0x0F));
        self.reg.set_cf(u16::from(a) < u16::from(val));
        self.reg.write8(Reg8::A, result);
    }

    /// ALU 8-bit subtract carry operation.
    /// Subtract a 8-bit value from a 8-bit register from a 8-bit register A with carry. (A = A - val - C).
    /// Flags: Z 1 H C
    fn alu_sbcr8(&mut self, reg: Reg8) {
        let a = self.reg.read8(Reg8::A);
        let val = self.reg.read8(reg);
        let c = if self.reg.cf() { 1 } else { 0 };
        let result = a.wrapping_sub(val).wrapping_sub(c);
        self.reg.set_zf(result == 0);
        self.reg.set_nf(true);
        self.reg.set_hf((a & 0x0F) < (val & 0x0F) + c);
        self.reg
            .set_cf(u16::from(a) < u16::from(result) + u16::from(c));
        self.reg.write8(Reg8::A, result);
    }

    /// ALU 8-bit subtract carry operation.
    /// Subtract a 8-bit value from a 8-bit register A with carry. (A = A - val - C).
    /// Flags: Z 1 H C
    fn alu_sbc8(&mut self, val: u8) {
        let a = self.reg.read8(Reg8::A);
        let c = if self.reg.cf() { 1 } else { 0 };
        let result = a.wrapping_sub(val).wrapping_sub(c);
        self.reg.set_zf(result == 0);
        self.reg.set_nf(true);
        self.reg.set_hf((a & 0x0F) < (val & 0x0F) + c);
        self.reg
            .set_cf(u16::from(a) < u16::from(result) + u16::from(c));
        self.reg.write8(Reg8::A, result);
    }

    /// ALU 8-bit AND operation.
    /// Bitwise AND a 8-bit value from a 8-bit register with a 8-bit register A.
    /// Flags: Z 0 1 0
    fn alu_andr8(&mut self, reg: Reg8) {
        let a = self.reg.read8(Reg8::A);
        let val = self.reg.read8(reg);
        let result = a & val;
        self.reg.set_zf(result == 0);
        self.reg.set_nf(false);
        self.reg.set_hf(true);
        self.reg.set_cf(false);
        self.reg.write8(Reg8::A, result);
    }

    /// ALU 8-bit AND operation.
    /// Bitwise AND a 8-bit value with a 8-bit register A.
    /// Flags: Z 0 1 0
    fn alu_and8(&mut self, val: u8) {
        let a = self.reg.read8(Reg8::A);
        let result = a & val;
        self.reg.set_zf(result == 0);
        self.reg.set_nf(false);
        self.reg.set_hf(true);
        self.reg.set_cf(false);
        self.reg.write8(Reg8::A, result);
    }

    /// ALU 8-bit XOR operation.
    /// Bitwise XOR a 8-bit value from a 8-bit register with a 8-bit register A.
    /// Flags: Z 0 0 0
    fn alu_xorr8(&mut self, reg: Reg8) {
        let a = self.reg.read8(Reg8::A);
        let val = self.reg.read8(reg);
        let result = a ^ val;
        self.reg.set_zf(result == 0);
        self.reg.set_nf(false);
        self.reg.set_hf(false);
        self.reg.set_cf(false);
        self.reg.write8(Reg8::A, result);
    }

    /// ALU 8-bit XOR operation.
    /// Bitwise XOR a 8-bit value with a 8-bit register A.
    /// Flags: Z 0 0 0
    fn alu_xor8(&mut self, val: u8) {
        let a = self.reg.read8(Reg8::A);
        let result = a ^ val;
        self.reg.set_zf(result == 0);
        self.reg.set_nf(false);
        self.reg.set_hf(false);
        self.reg.set_cf(false);
        self.reg.write8(Reg8::A, result);
    }

    /// ALU 8-bit OR operation.
    /// Bitwise OR a 8-bit value from a 8-bit register with a 8-bit register A.
    /// Flags: Z 0 0 0
    fn alu_orr8(&mut self, reg: Reg8) {
        let a = self.reg.read8(Reg8::A);
        let val = self.reg.read8(reg);
        let result = a | val;
        self.reg.set_zf(result == 0);
        self.reg.set_nf(false);
        self.reg.set_hf(false);
        self.reg.set_cf(false);
        self.reg.write8(Reg8::A, result);
    }

    /// ALU 8-bit OR operation.
    /// Bitwise OR a 8-bit value with a 8-bit register A.
    /// Flags: Z 0 0 0
    fn alu_or8(&mut self, val: u8) {
        let a = self.reg.read8(Reg8::A);
        let result = a | val;
        self.reg.set_zf(result == 0);
        self.reg.set_nf(false);
        self.reg.set_hf(false);
        self.reg.set_cf(false);
        self.reg.write8(Reg8::A, result);
    }

    /// ALU 8-bit compare operation.
    /// Compare a 8-bit value from a 8-bit register with a 8-bit register A.
    /// Flags: Z 1 H C
    fn alu_cpr8(&mut self, reg: Reg8) {
        let a = self.reg.read8(Reg8::A);
        let val = self.reg.read8(reg);
        let result = a.wrapping_sub(val);
        self.reg.set_zf(result == 0);
        self.reg.set_nf(true);
        self.reg.set_hf((a & 0x0F) < (val & 0x0F));
        self.reg.set_cf(u16::from(a) < u16::from(result));
    }

    /// ALU 8-bit compare operation.
    /// Compare a 8-bit value with a 8-bit register A.
    /// Flags: Z 1 H C
    fn alu_cp8(&mut self, val: u8) {
        let a = self.reg.read8(Reg8::A);
        let result = a.wrapping_sub(val);
        self.reg.set_zf(result == 0);
        self.reg.set_nf(true);
        self.reg.set_hf((a & 0x0F) < (val & 0x0F));
        self.reg.set_cf(u16::from(a) < u16::from(result));
    }

    /// ALU DAA operation.
    /// Decimal adjust register A.
    /// This instruction adjusts register A so that the correct representation of Binary Coded Decimal (BCD) is obtained.
    /// Flags: Z 0 H C
    /// General DAA implementation - https://www.scs.stanford.edu/nyu/04fa/lab/i386/DAA.htm
    /// Implementation pulled from AWJ's post #433 here - https://forums.nesdev.org/viewtopic.php?f=20&t=15944
    /// thank you <3
    /// NOTE: If this fails, it is probably due to how H and N flags are set in the other instructions.
    ///       DAA is the only thing that actually uses those flags!
    ///
    ///    // note: assumes a is a uint8_t and wraps from 0xff to 0
    ///    if (!n_flag) {  // after an addition, adjust if (half-)carry occurred or if result is out of bounds
    ///    if (c_flag || a > 0x99) { a += 0x60; c_flag = 1; }
    ///    if (h_flag || (a & 0x0f) > 0x09) { a += 0x6; }
    ///    } else {  // after a subtraction, only adjust if (half-)carry occurred
    ///    if (c_flag) { a -= 0x60; }
    ///    if (h_flag) { a -= 0x6; }
    ///    }
    ///   // these flags are always updated
    ///    z_flag = (a == 0); // the usual z flag
    ///    h_flag = 0; // h flag is always cleared
    fn alu_daa(&mut self) {
        let mut a = self.reg.read8(Reg8::A);
        let mut adjust = 0;
        if self.reg.hf() || (!self.reg.nf() && (a & 0x0F) > 0x09) {
            adjust |= 0x06;
        }
        if self.reg.cf() || (!self.reg.nf() && a > 0x99) {
            adjust |= 0x60;
            self.reg.set_cf(true);
        }
        if self.reg.nf() {
            a = a.wrapping_sub(adjust);
        } else {
            a = a.wrapping_add(adjust);
        }
        self.reg.set_zf(a == 0);
        self.reg.set_hf(false);
        self.reg.write8(Reg8::A, a);
    }

    /// ALU CPL operation.
    /// Complement register A (Flip all bits).
    /// Flags: - 1 1 -
    fn alu_cpl(&mut self) {
        let a = !self.reg.read8(Reg8::A);
        self.reg.set_nf(true);
        self.reg.set_hf(true);
        self.reg.write8(Reg8::A, a);
    }

    /// ALU SCF operation.
    /// Set carry flag.
    /// Flags: - 0 0 1
    fn alu_scf(&mut self) {
        self.reg.set_nf(false);
        self.reg.set_hf(false);
        self.reg.set_cf(true);
    }

    /// ALU CCF operation.
    /// Complement carry flag.
    /// If the carry flag is set, then it is reset. Otherwise, it is set. (CF = !CF).
    /// Flags: - 0 0 C
    fn alu_ccf(&mut self) {
        self.reg.set_nf(false);
        self.reg.set_hf(false);
        self.reg.set_cf(!self.reg.cf());
    }

    /// ALU Shift/Rotate Update Flags
    /// Update flags for shift/rotate operations.
    /// Flags: Z 0 0 C
    fn alu_sr_flags(&mut self, val: u8, carry: bool) {
        self.reg.set_zf(val == 0);
        self.reg.set_nf(false);
        self.reg.set_hf(false);
        self.reg.set_cf(carry);
    }

    /// ALU Rotate Left carry operation.
    /// Rotate an 8-bit value left through carry flag, return result.
    fn alu_rlc(&mut self, val: u8) -> u8 {
        let carry = (val & 0x80) == 0x80;
        let result = (val << 1) | (if carry { 1 } else { 0 });
        self.alu_sr_flags(result, carry);
        result
    }

    /// ALU Rotate Left operation.
    /// Rotate an 8-bit value left, return result.
    fn alu_rl(&mut self, val: u8) -> u8 {
        let carry = (val & 0x80) == 0x80;
        let result = (val << 1) | (if self.reg.cf() { 1 } else { 0 });
        self.alu_sr_flags(result, carry);
        result
    }

    /// ALU Rotate Right carry operation.
    /// Rotate an 8-bit value right through carry flag, return result.
    fn alu_rrc(&mut self, val: u8) -> u8 {
        let carry = (val & 0x01) == 0x01;
        let result = if carry { 0x80 | (val >> 1) } else { val >> 1 };
        self.alu_sr_flags(result, carry);
        result
    }

    /// ALU Rotate Right operation.
    /// Rotate an 8-bit value right, return result.
    fn alu_rr(&mut self, val: u8) -> u8 {
        let carry = (val & 0x01) == 0x01;
        let result = if self.reg.cf() {
            0x80 | (val >> 1)
        } else {
            val >> 1
        };
        self.alu_sr_flags(result, carry);
        result
    }

    /// ALU Shift Left operation.
    /// Shift an 8-bit value left, into carry, return result. LSB is set to 0.
    /// Flags: Z 0 0 C
    fn alu_sla(&mut self, val: u8) -> u8 {
        let carry = (val & 0x80) == 0x80;
        let result = val << 1;
        self.alu_sr_flags(result, carry);
        result
    }

    /// ALU Shift Right operation.
    /// Shift an 8-bit value right, into carry, return result. MSB is unchanged.
    /// Flags: Z 0 0 C
    fn alu_sra(&mut self, val: u8) -> u8 {
        let carry = (val & 0x01) == 0x01;
        let result = (val & 0x80) | (val >> 1);
        self.alu_sr_flags(result, carry);
        result
    }

    /// ALU Shift Right operation.
    /// Shift an 8-bit value right, into carry, return result. MSB is set to 0.
    /// Flags: Z 0 0 C
    fn alu_srl(&mut self, val: u8) -> u8 {
        let carry = (val & 0x01) == 0x01;
        let result = val >> 1;
        self.alu_sr_flags(result, carry);
        result
    }

    /// ALU Swap operation.
    /// Swap upper and lower nibbles of an 8-bit value, return result.
    /// Flags: Z 0 0 0
    fn alu_swap(&mut self, val: u8) -> u8 {
        self.reg.set_zf(val == 0);
        self.reg.set_nf(false);
        self.reg.set_hf(false);
        self.reg.set_cf(false);
        (val >> 4) | (val << 4)
    }

    /// ALU Bit Test operation.
    /// Test bit b in value r (usually a register). Set Z flag if bit is 0.
    /// Flags: Z 0 1 -
    fn alu_bit(&mut self, b: u8, r: u8) {
        let result = r & (1 << b) == 0x00;
        self.reg.set_zf(result);
        self.reg.set_nf(false);
        self.reg.set_hf(true);
    }

    /// ALU Bit Reset operation.
    /// Reset bit b in value r (usually a register).
    /// Flags: None
    fn alu_res(&mut self, b: u8, r: u8) -> u8 {
        r & !(1 << b)
    }

    /// ALU Bit Set operation.
    /// Set bit b in value r (usually a register).
    /// Flags: None
    fn alu_set(&mut self, b: u8, r: u8) -> u8 {
        r | (1 << b)
    }
}
