pub mod clock;

use std::{cell::RefCell, rc::Rc};

use crate::cpu::interrupts::{Flags, InterruptFlags};

use self::clock::Clock;

#[derive(Default)]
struct Register {
    // This register is incremented at rate of 16384Hz (~16779Hz on SGB). Writing any value to this register resets it
    // to 00h.
    // Note: The divider is affected by CGB double speed mode, and will increment at 32768Hz in double speed.
    div: u8,
    // This timer is incremented by a clock frequency specified by the TAC register ($FF07). When the value overflows
    // (gets bigger than FFh) then it will be reset to the value specified in TMA (FF06), and an interrupt will be
    // requested, as described below.
    tima: u8,
    // When the TIMA overflows, this data will be loaded.
    tma: u8,
    //  Bit  2   - Timer Enable
    //  Bits 1-0 - Input Clock Select
    //             00: CPU Clock / 1024 (DMG, CGB:   4096 Hz, SGB:   ~4194 Hz)
    //             01: CPU Clock / 16   (DMG, CGB: 262144 Hz, SGB: ~268400 Hz)
    //             10: CPU Clock / 64   (DMG, CGB:  65536 Hz, SGB:  ~67110 Hz)
    //             11: CPU Clock / 256  (DMG, CGB:  16384 Hz, SGB:  ~16780 Hz)
    tac: u8,
}

// Each time when the timer overflows (ie. when TIMA gets bigger than FFh), then an interrupt is requested by
// setting Bit 2 in the IF Register (FF0F). When that interrupt is enabled, then the CPU will execute it by calling
// the timer interrupt vector at 0050h.
pub struct Timer {
    if_: Rc<RefCell<InterruptFlags>>,
    reg: Register,
    div_clock: Clock,
    tma_clock: Clock,
}

impl Timer {
    pub fn new(if_: Rc<RefCell<InterruptFlags>>) -> Self {
        Timer {
            if_,
            reg: Register::default(),
            div_clock: Clock::new(256),
            tma_clock: Clock::new(1024),
        }
    }

    pub fn get(&self, a: u16) -> u8 {
        match a {
            0xff04 => self.reg.div,
            0xff05 => self.reg.tima,
            0xff06 => self.reg.tma,
            0xff07 => self.reg.tac,
            _ => panic!("Unsupported address"),
        }
    }

    pub fn set(&mut self, a: u16, v: u8) {
        match a {
            0xff04 => {
                self.reg.div = 0x00;
                self.div_clock.n = 0x00;
            }
            0xff05 => self.reg.tima = v,
            0xff06 => self.reg.tma = v,
            0xff07 => {
                if (self.reg.tac & 0x03) != (v & 0x03) {
                    self.tma_clock.n = 0x00;
                    self.tma_clock.period = match v & 0x03 {
                        0x00 => 1024,
                        0x01 => 16,
                        0x02 => 64,
                        0x03 => 256,
                        _ => panic!(""),
                    };
                    self.reg.tima = self.reg.tma;
                }
                self.reg.tac = v;
            }
            _ => panic!("Unsupported address"),
        }
    }

    pub fn cycle(&mut self, cycles: u32) {
        // Increment div at rate of 16384Hz. Because the clock cycles is 4194304, so div increment every 256 cycles (4194304/256).
        self.reg.div = self
            .reg
            .div
            .wrapping_add(self.div_clock.cycle(cycles) as u8);

        // Timer Enabled?
        if (self.reg.tac & 0x04) != 0x00 {
            // Increment tima at rate of Clock / freq
            let n = self.tma_clock.cycle(cycles);
            for _ in 0..n {
                self.reg.tima = self.reg.tima.wrapping_add(1);
                if self.reg.tima == 0x00 {
                    self.reg.tima = self.reg.tma;
                    self.if_.borrow_mut().set(Flags::Timer);
                }
            }
        }
    }
}
