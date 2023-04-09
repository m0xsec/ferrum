mod registers;

/// The DMG-01 had a Sharp LR35902 CPU (speculated to be a SM83 core), which is a hybrid of the Z80 and the 8080
/// https://gbdev.io/gb-opcodes/optables/errata
pub struct CPU {
    // Registers
    reg: registers::Registers,
    // Memory
    // TODO: Pointer reference to GB MMU

    // Clock Cycles
    // Interesting discussion - https://www.reddit.com/r/EmuDev/comments/4o2t6k/how_do_you_emulate_specific_cpu_speeds/
    //cycles    uint32
    //maxCycles uint32
    // TODO: Keep track of clock cycles

    // Halt flag
    // bool
    // TODO: CPU Instruction Halt flag
}

impl CPU {
    /// Initialize the CPU
    pub(crate) fn new() -> Self {
        Self {
            reg: registers::Registers::new(),
        }
    }
}
