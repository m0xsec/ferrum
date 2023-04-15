use super::{
    opcodes,
    registers::{Reg16, Reg8},
    CPU,
};
use log::info;
use std::collections::HashMap;

impl CPU {
    /// Executes a CPU operation, returns the number of cycles
    pub fn op_execute(&mut self, op: u8) -> (u8, u32) {
        let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODES_MAP;
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

            _ => {
                todo!("opcode: {:#02x}.", op);
            }
        }

        (opcode.length, opcode.cycles)
    }
}

impl CPU {
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
