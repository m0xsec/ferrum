use log::{info, warn};

use crate::mmu::memory::Memory;
use std::cell::RefCell;
use std::rc::Rc;
mod execute;
pub mod interrupts;
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
            ime: true,

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

    /// Handles CPU Interrupts and returns the number of cycles the interrupt took.
    pub fn handle_interrupts(&mut self) -> u32 {
        // Interrupts are handled by the CPU, not the MMU.
        // The IME (interrupt master enable) flag is reset by DI and prohibits all interrupts. It is set by EI and
        // acknowledges the interrupt setting by the IE register.
        // 1. When an interrupt is generated, the IF flag will be set.
        // 2. If the IME flag is set & the corresponding IE flag is set, the following 3 steps are performed.
        // 3. Reset the IME flag and prevent all interrupts.
        // 4. The PC (program counter) is pushed onto the stack.
        // 5. Jump to the starting address of the interrupt.

        // If CPU is halted and interrupts are disabled, do nothing.
        if self.halt && !self.ime {
            return 0;
        }

        // Get Interrupt Enable and Interrupt Flag registers
        let ie = self.mem.borrow().read8(0xFFFF);
        let if_ = self.mem.borrow().read8(0xFF0F);

        // If interrupts are disabled, do nothing.
        if !self.ime {
            return 0;
        }

        // If no interrupts are enabled, do nothing.
        if ie == 0x00 {
            return 0;
        }

        // If no interrupts are pending, do nothing.
        if if_ == 0x00 {
            return 0;
        }

        // If interrupts are enabled, but none are pending, do nothing.
        if ie & if_ == 0x00 {
            return 0;
        }

        // If we get here, we have an interrupt to handle.
        // Reset IME and CPU halt.
        self.ime = false;
        self.halt = false;

        // Push the current PC onto the stack
        let pc = self.reg.read16(registers::Reg16::PC);
        self.stack_push(pc);

        // Consume the interrupt, and write the remaining interrupts back to the IF register.
        let i = (if_ & ie).trailing_zeros();
        self.mem.borrow_mut().write8(0xFF0F, if_ & !(1 << i));

        // Jump to the interrupt
        let i_addr = match i {
            // V-Blank
            0 => 0x0040,
            // LCD STAT
            1 => 0x0048,
            // Timer
            2 => 0x0050,
            // Serial
            3 => 0x0058,
            // Joypad
            4 => 0x0060,
            _ => panic!("Invalid interrupt!"),
        };
        self.reg.write16(registers::Reg16::PC, i_addr);

        4
    }

    /// Cycle the CPU for a single instruction - Fetch, decode, execute
    pub fn cycle(&mut self) {
        // Handle interrupts
        self.cycles += self.handle_interrupts();

        // If CPU is halted, do nothing.
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
