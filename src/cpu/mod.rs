use log::{info, warn};

use crate::mmu::memory::Memory;
use std::cell::RefCell;
use std::rc::Rc;
mod execute;
mod opcodes;
mod registers;

/// The DMG-01 had a Sharp LR35902 CPU (speculated to be a SM83 core), which is a hybrid of the Z80 and the 8080
/// https://gbdev.io/gb-opcodes/optables/errata
pub struct Cpu {
    /// Registers
    reg: registers::Registers,

    /// Memory
    mem: Rc<RefCell<dyn Memory>>,

    /// Interrupt Master Enable Flag (IME)
    ime: bool,

    /// Clock Cycles
    /// Interesting discussion - https://www.reddit.com/r/EmuDev/comments/4o2t6k/how_do_you_emulate_specific_cpu_speeds/
    /// 4.194304 MHz was the highest freq the DMG could run at.
    cycles: u32,
    max_cycles: u32,

    /// Halt flag, for stopping CPU operation.
    halt: bool,
}

impl Cpu {
    /// Fetches the next opcode from memory
    fn fetch(&self) -> u8 {
        self.mem
            .borrow()
            .read8(self.reg.read16(registers::Reg16::PC))
    }
}

impl Cpu {
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
            ime: false,

            // 4.194304 MHz was the highest freq the DMG could run at.
            cycles: 0,
            max_cycles: 4194304,

            halt: false,
        }
    }

    /// When we are skipping the Boot ROM for testing, set the registers to what the boot ROM would set them to normally.
    pub fn test_set_boot_regs(&mut self) {
        info!("Setting registers to boot ROM values (for testing)...");
        self.reg.write8(registers::Reg8::A, 0x01);
        self.reg.write8(registers::Reg8::F, 0xB0);
        self.reg.write8(registers::Reg8::B, 0x00);
        self.reg.write8(registers::Reg8::C, 0x13);
        self.reg.write8(registers::Reg8::D, 0x00);
        self.reg.write8(registers::Reg8::E, 0xD8);
        self.reg.write8(registers::Reg8::H, 0x01);
        self.reg.write8(registers::Reg8::L, 0x4D);
        self.reg.write16(registers::Reg16::PC, 0x0100);
        self.reg.write16(registers::Reg16::SP, 0xFFFE);
    }

    /// Blargg test ROMs will output results to the serial port as an alternative to the PPU.
    pub fn test_read_blargg_serial(&self) {
        /*
        Everything printed on screen is also sent to the game link port by
        writing the character to SB, then writing $81 to SC. This is useful for
        tests which print lots of information that scrolls off screen.
        */

        // Character is written to SB, then $81 is written to SC.
        let sb = self.mem.borrow().read8(0xFF01);
        let sc = self.mem.borrow().read8(0xFF02);
        if sc == 0x81 {
            print!("{}", sb as char);
            self.mem.borrow_mut().write8(0xFF02, 0x00);
        }
    }

    /// Cycle the CPU for a single instruction - Fetch, decode, execute
    pub fn cycle(&mut self) {
        if !self.halt {
            let op = self.fetch();
            let (len, cycle) = self.op_execute(op);
            self.reg.inc_pc(len.into());
            self.cycles += cycle;
        } else {
            info!("CPU halted!");
        }

        if self.cycles > self.max_cycles {
            warn!("Max CPU Cycles detected, though not yet implemented.");
            info!("Enforcing 4.194304 Mhz");
            // TODO: Sleep for 1 second
            self.cycles = 0;
        }
    }

    /// Dumps the current CPU Register values at the info Log level.
    pub fn dump_registers(&self) {
        info!("CPU Registers{}", self.reg);
    }
}
