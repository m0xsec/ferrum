mod registers;

/// The DMG-01 had a Sharp LR35902 CPU (speculated to be a SM83 core), which is a hybrid of the Z80 and the 8080
/// https://gbdev.io/gb-opcodes/optables/errata
pub struct CPU {
    /// Registers
    reg: registers::Registers,
    // Memory
    // TODO: Pointer reference to GB MMU
    // TODO: Look into implementing a trait for memory... https://github.com/mohanson/gameboy/blob/master/src/mmunit.rs
    /// Clock Cycles
    /// Interesting discussion - https://www.reddit.com/r/EmuDev/comments/4o2t6k/how_do_you_emulate_specific_cpu_speeds/
    /// 4.194304 MHz was the highest freq the DMG could run at.
    cycles: u32,
    max_cycle: u32,

    /// Halt flag, for stopping CPU operation.
    halt: bool,
}

impl CPU {
    /// Initialize the CPU
    pub fn new() -> Self {
        Self {
            /*
                Set initial registers to 0x00 - The DMG-01 power up sequence, per PanDocs, is:
                https://gbdev.io/pandocs/Power_Up_Sequence.html
                A = 0x01
                F = 0xB0
                B = 0x00
                C = 0x13
                D = 0x00
                E = 0xD8
                H = 0x01
                L = 0x4D
                PC = 0x0100
                SP = 0xFFFE

                This should be what the boot ROM does.
            */
            reg: registers::Registers::new(),

            // 4.194304 MHz was the highest freq the DMG could run at.
            cycles: 0,
            max_cycle: 4194304,

            halt: false,
        }
    }
}
