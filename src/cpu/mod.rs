use log::info;

use crate::mmu::memory::Memory;
use std::cell::RefCell;
use std::rc::Rc;
pub mod registers;

/// The DMG-01 had a Sharp LR35902 CPU (speculated to be a SM83 core), which is a hybrid of the Z80 and the 8080
/// https://gbdev.io/gb-opcodes/optables/errata
pub struct CPU {
    /// Registers
    pub reg: registers::Registers,

    /// Memory
    mem: Rc<RefCell<dyn Memory>>,

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
    pub fn power_on(mem: Rc<RefCell<dyn Memory>>) -> Self {
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
            mem,

            // 4.194304 MHz was the highest freq the DMG could run at.
            cycles: 0,
            max_cycle: 4194304,

            halt: false,
        }
    }

    /// NOTE: This is for testing prohibited memory operations on the MMU. For debugging only.
    pub fn prohibited_memory_operation_test(&mut self) {
        self.mem.borrow_mut().write(0xFEA0, 0x2C);
    }

    /// Dumps the current CPU Register values at the info Log level.
    pub fn dump_registers(&self) {
        info!("CPU Registers{}", self.reg);
    }
}
