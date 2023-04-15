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

        info!("{}", &opcode.mnemonic);

        match op {
            0x00 => {
                // NOP
            }

            // LD r16, d16
            0x01 | 0x11 | 0x21 | 0x31 => {
                let val = self.imm16();
                match op {
                    // 0x01 - LD BC, d16 - Load 16-bit immediate value d16 into register BC
                    0x01 => self.ld16(Reg16::BC, val),

                    // 0x11 - LD DE, d16 - Load 16-bit immediate value d16 into register DE
                    0x11 => self.ld16(Reg16::DE, val),

                    // 0x21 - LD HL, d16 - Load 16-bit immediate value d16 into register HL
                    0x21 => self.ld16(Reg16::HL, val),

                    // 0x31 - LD SP, d16 - Load 16-bit immediate value d16 into register SP
                    0x31 => self.ld16(Reg16::SP, val),
                    _ => {}
                }
            }

            // LD (BC), A - Load A into memory at address BC
            0x02 => {
                self.ld8(self.reg.read16(Reg16::BC), self.reg.read8(Reg8::A));
            }

            _ => {
                todo!("opcode: {:#02x}.", op);
            }
        }

        (opcode.length, opcode.cycles)
    }
}

impl CPU {
    /// Fetch the immediate byte (u8)
    fn imm8(&mut self) -> u8 {
        self.mem.borrow().read8(self.reg.read16(Reg16::PC))
    }

    /// Fetch the immediate word (u16)
    fn imm16(&mut self) -> u16 {
        self.mem.borrow().read16(self.reg.read16(Reg16::PC))
    }

    /// 8-bit load operation.
    /// Load an 8-bit value (val) into the 16-bit address (dst).
    fn ld8(&mut self, dst: u16, val: u8) {
        self.mem.borrow_mut().write8(dst, val);
    }

    /// 16-bit load operation.
    /// Load a 16-bit value (val) into the 16-bit register (dst).
    fn ld16(&mut self, dst: Reg16, val: u16) {
        self.reg.write16(dst, val);
    }
}
