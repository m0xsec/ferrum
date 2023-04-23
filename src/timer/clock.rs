/// Every clock tick occurs 1 cycle every N cycles.
pub struct Clock {
    pub period: u32,
    pub n: u32,
}

impl Clock {
    pub fn new(period: u32) -> Self {
        Self { period, n: 0x00 }
    }

    /// Returns the number of ticks that have occurred
    pub fn cycle(&mut self, cycles: u32) -> u32 {
        self.n += cycles;
        let ticks = self.n / self.period;
        self.n %= self.period;
        ticks
    }
}
