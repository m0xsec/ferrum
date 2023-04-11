/*
TODO: For each opcode, find a nice way to keep track of name, cycles, and the execution function
Checkout this guide and code for a Rust NES emulator -
https://bugzmanov.github.io/nes_ebook/chapter_3_4.html
https://github.com/bugzmanov/nes_ebook/blob/master/code/ch3.4/src/opcodes.rs

This is exactly what I had in mind.
 */

use std::collections::HashMap;

pub struct OpCode {
    pub op: u8,
    pub mnemonic: &'static str,
    pub cycles: u32,
}

impl OpCode {
    fn new(op: u8, mnemonic: &'static str, cycles: u32) -> Self {
        OpCode {
            op: op,
            mnemonic: mnemonic,
            cycles: cycles,
        }
    }
}

lazy_static! {
    pub static ref CPU_OP_CODES: Vec<OpCode> = vec![
        OpCode::new(0x00, "NOP", 4),
        OpCode::new(0x01, "NOP", 4),
        OpCode::new(0x02, "NOP", 4),
    ];
    pub static ref OPCODES_MAP: HashMap<u8, &'static OpCode> = {
        let mut map = HashMap::new();
        for cpu_op in &*CPU_OP_CODES {
            map.insert(cpu_op.op, cpu_op);
        }
        map
    };
}
