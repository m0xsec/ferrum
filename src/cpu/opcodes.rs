use std::collections::HashMap;

pub struct OpCode {
    /// CPU Instruction, represented as a hexadecimal u8.
    /// For example, 0x00.
    pub op: u8,

    /// Instruction mnemonic. For example "NOP".
    pub mnemonic: &'static str,

    /// The length in bytes. For example, 4.
    pub length: u8,

    /// Duration in cycles.
    /// Our definition of "cycle" is based on system clock ticks, or T-states.
    pub cycles: u32,
}

impl OpCode {
    fn new(op: u8, mnemonic: &'static str, length: u8, cycles: u32) -> Self {
        OpCode {
            op: op,
            mnemonic: mnemonic,
            length: length,
            cycles: cycles,
        }
    }
}

lazy_static! {
    //TODO: Add all Z80 / SM83 / LR35902 CPU instructions for the Game Boy...
    pub static ref CPU_OP_CODES: Vec<OpCode> = vec![
        OpCode::new(0x00, "NOP", 1, 4),
        OpCode::new(0x01, "LD BC, d16", 3, 12),
        OpCode::new(0x02, "LD (BC), A", 1, 8),
        OpCode::new(0x03, "INC BC", 1, 8),


        // TODO: Add an additional ref for all of the PREFIX CB CPU ops...

    ];
    pub static ref OPCODES_MAP: HashMap<u8, &'static OpCode> = {
        let mut map = HashMap::new();
        for cpu_op in &*CPU_OP_CODES {
            map.insert(cpu_op.op, cpu_op);
        }
        map
    };
}
