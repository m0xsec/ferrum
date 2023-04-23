pub trait Memory {
    /// Read a byte (u8) from memory.
    fn read8(&self, addr: u16) -> u8;

    /// Write a byte (u8) to memory.
    fn write8(&mut self, addr: u16, val: u8);

    /// Read a word (u16) from memory.
    fn read16(&self, addr: u16) -> u16;

    /// Write a word (u16) to memory.
    fn write16(&mut self, addr: u16, val: u16);

    /// Cycle the memory.
    fn cycle(&mut self, ticks: u32) -> u32;
}
