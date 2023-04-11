use super::{opcodes, CPU};
use log::{info, warn};
use std::collections::HashMap;

impl CPU {
    /// Executes a CPU operation, returns the number of cycles
    pub fn op_execute(&mut self, op: u8) -> u32 {
        let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODES_MAP;
        let opcode = opcodes.get(&op).unwrap_or(&&opcodes::OpCode {
            op: 0x00,
            mnemonic: "NOT IMPLEMENTED",
            cycles: 0,
        });

        match op {
            0x00 => {
                info!("{}", &opcode.mnemonic);
                // Preform opcode execution logic here...
                self.reg.inc_pc(1);
                opcode.cycles
            }
            _ => {
                //todo!();
                warn!("opcode not implemented: {:#02x}.", op);
                self.reg.inc_pc(1);
                0
            }
        }
    }
}
