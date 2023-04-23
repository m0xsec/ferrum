use log::{info, warn};

use crate::mmu::memory::Memory;
use std::cell::RefCell;
use std::rc::Rc;
use std::{thread, time};
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

    /// Keeps track of the Boot ROM being enabled or disabled.
    boot_rom_enabled: bool,

    /// Interrupt Master Enable Flag (IME)
    ime: bool,

    /// Halt flag, for stopping CPU operation.
    halt: bool,
}

impl Cpu {
    /// Fetches the next opcode from memory
    fn fetch(&mut self) -> u8 {
        /*self.mem
        .borrow()
        .read8(self.reg.read16(registers::Reg16::PC))*/
        self.imm8()
    }

    /// Handles CPU Interrupts and returns the number of cycles the interrupt took.
    fn handle_interrupts(&mut self) -> u32 {
        // Interrupts are handled by the CPU, not the MMU.
        // The IME (interrupt master enable) flag is reset by DI and prohibits all interrupts. It is set by EI and
        // acknowledges the interrupt setting by the IE register.
        // 1. When an interrupt is generated, the IF flag will be set.
        // 2. If the IME flag is set & the corresponding IE flag is set, the following 3 steps are performed.
        // 3. Reset the IME flag and prevent all interrupts.
        // 4. The PC (program counter) is pushed onto the stack.
        // 5. Jump to the starting address of the interrupt.

        // If CPU is halted and interrupts are disabled, do nothing.
        if !self.halt && !self.ime {
            return 0;
        }

        // Get Interrupt Enable and Interrupt Flag registers
        let ie = self.mem.borrow().read8(0xFFFF);
        let if_ = self.mem.borrow().read8(0xFF0F);
        let triggered = ie & if_;

        // If interrupts are enabled, but none are pending, do nothing.
        if triggered == 0x00 {
            return 0;
        }

        // If we get here, we have an interrupt to handle.
        // Reset IME and CPU halt.
        self.halt = false;

        if !self.ime {
            return 0;
        }
        self.ime = false;

        // Consume the interrupt, and write the remaining interrupts back to the IF register.
        let i = triggered.trailing_zeros();
        self.mem.borrow_mut().write8(0xFF0F, if_ & !(1 << i));

        // Push the current PC onto the stack
        let pc = self.reg.read16(registers::Reg16::PC);
        self.stack_push(pc);

        // Jump to the interrupt
        self.reg
            .write16(registers::Reg16::PC, 0x0040 | ((i as u16) << 3));

        4 * 4
    }

    /// Prints the current CPU state to the console.
    /// Following the format that Gameboy Logs repo uses
    /// https://github.com/wheremyfoodat/Gameboy-logs
    fn _debug_print_state(&self) {
        let pc = self.reg.read16(registers::Reg16::PC);
        let sp = self.reg.read16(registers::Reg16::SP);
        let a = self.reg.read8(registers::Reg8::A);
        let f = self.reg.read8(registers::Reg8::F);
        let b = self.reg.read8(registers::Reg8::B);
        let c = self.reg.read8(registers::Reg8::C);
        let d = self.reg.read8(registers::Reg8::D);
        let e = self.reg.read8(registers::Reg8::E);
        let h = self.reg.read8(registers::Reg8::H);
        let l = self.reg.read8(registers::Reg8::L);
        let m = self.mem.borrow().read8(pc);
        let n = self.mem.borrow().read8(pc + 1);
        let o = self.mem.borrow().read8(pc + 2);
        let p = self.mem.borrow().read8(pc + 3);

        // Print using the following format
        // [registers] (mem[pc] mem[pc+1] mem[pc+2] mem[pc+3])
        println!(
            "A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X} E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: 00:{:04X} ({:02X} {:02X} {:02X} {:02X})",
            a, f, b, c, d, e, h, l, sp, pc, m, n, o, p
        );
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
            boot_rom_enabled: true,
            ime: false,
            halt: false,
        }
    }

    /// Cycle the CPU for a single instruction - Fetch, decode, execute
    pub fn cycle(&mut self) -> u32 {
        //self._debug_print_state();
        let mut ticks = 0;

        // If CPU is halted, do nothing.
        if !self.halt {
            let op = self.fetch();
            ticks += self.op_execute(op);
        } else {
            info!("CPU halted!");
            ticks += 1;
        }

        ticks += self.handle_interrupts();
        //println!("Ticks: {}", ticks);
        self.mem.borrow_mut().cycle(ticks)
    }

    /// Dumps the current CPU Register values at the info Log level.
    pub fn dump_registers(&self) {
        info!("CPU Registers{}", self.reg);
    }
}
