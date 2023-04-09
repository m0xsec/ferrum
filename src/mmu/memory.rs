pub trait Memory {
    /// Read a value from memory.
    fn read(&self, addr: u16) -> u8;
    /// Write a value from memory.
    fn write(&mut self, addr: u16, val: u8);
}
