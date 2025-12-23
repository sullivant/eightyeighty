use crate::memory::{self, Memory};


// For mapping I/O devices
pub trait IoDevice {
    fn input(&mut self, port: u8) -> u8;
    fn output(&mut self, port: u8, value: u8);
}

// Null Device will do... nothing.
pub struct NullDevice;

impl IoDevice for NullDevice {
    fn input(&mut self, _port: u8) -> u8 {
        0
    }
    fn output(&mut self, _port: u8, _value: u8) {
        // Does nothing.
    }
}

pub struct Bus {
    memory: Memory,
    io: Box<dyn IoDevice>,
}

impl Bus {
    // Initial bus creation has no mapped IO device
    #[must_use]
    pub fn new(memory: Memory) -> Self{
        Self {
            memory,
            io: Box::new(NullDevice), // No real device to start
        }
    }

    // Create a bus with an IO device if wanted
    #[must_use]
    pub fn with_io(memory: Memory, io: Box<dyn IoDevice>) -> Self {
        Self { memory, io }
    }

    // Memory related stuff
    #[inline]
    #[must_use]
    pub fn read(&self, addr: usize) -> u8 {
        self.memory.read(addr).unwrap_or_default()
    }

    #[inline]
    pub fn write(&mut self, addr: usize, value: u8) {
        self.memory.write(addr, value);
    }

    // Allows larger access
    #[must_use]
    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut Memory {
        &mut self.memory
    }

    // IO things
    #[inline]
    pub fn input(&mut self, port: u8) -> u8 {
        self.io.input(port)
    }

    #[inline]
    pub fn output(&mut self, port: u8, value: u8) {
        self.io.output(port, value);
    }
}
