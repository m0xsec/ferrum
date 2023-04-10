/*
TODO: For each opcode, find a nice way to keep track of name, cycles, and the execution function
 */

use log::{info, warn};

use super::registers::{Reg16, Reg8};
use super::CPU;

impl CPU {
    /// Executes a CPU operation, returns the number of cycles
    pub fn execute(&mut self, op: u8) -> u32 {
        match op {
            0x00 => {
                info!("NOP");
                self.reg.inc_pc(1);
                4
            }
            _ => {
                warn!("opcode not implemented: {:#02x}.", op);
                self.reg.inc_pc(1);
                0
            }
        }
    }
}
