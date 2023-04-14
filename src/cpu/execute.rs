use super::{opcodes, CPU};
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
                // TODO: Preform opcode execution logic here...
            }

            _ => {
                todo!("opcode: {:#02x}.", op);
            }
        }

        (opcode.length, opcode.cycles)
    }
}
