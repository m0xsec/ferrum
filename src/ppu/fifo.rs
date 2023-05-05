/// FIFO (First In First Out) queue for storing pixel data.
/// This will be used for shifting pixel data out to the LCD.
/// This data structure is a fixed size.
struct Fifo {
    /// Array of values within the FIFO.
    data: [u8; 16],

    /// Index of the tail of the FIFO (output).
    tail: usize,

    /// Index of the head of the FIFO (input).
    head: usize,

    /// Number of elements in the FIFO.
    size: usize,
}

impl Fifo {
    /// Create a new FIFO.
    pub fn new() -> Fifo {
        Fifo {
            data: [0; 16],
            tail: 0,
            head: 0,
            size: 0,
        }
    }

    /// Push a value onto the FIFO.
    pub fn push(&mut self, value: u8) {
        if self.size == 16 {
            panic!("FIFO is full");
        }

        self.data[self.head] = value;
        self.head = (self.head + 1) % 16;
        self.size += 1;
    }

    /// Pop a value off of the FIFO.
    pub fn pop(&mut self) -> u8 {
        if self.size == 0 {
            panic!("FIFO is empty");
        }

        let value = self.data[self.tail];
        self.tail = (self.tail + 1) % 16;
        self.size -= 1;

        value
    }

    /// Get the number of elements in the FIFO.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Clear the FIFO.
    pub fn clear(&mut self) {
        self.tail = 0;
        self.head = 0;
        self.size = 0;
    }
}
