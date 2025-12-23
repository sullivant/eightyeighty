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
    pub fn new(memory: Memory) -> Self{
        Self {
            memory,
            io: Box::new(NullDevice), // No real device to start
        }
    }

    // Create a bus with an IO device if wanted
    pub fn with_io(memory: Memory, io: Box<dyn IoDevice>) -> Self {
        Self { memory, io }
    }

    // Memory related stuff
    #[inline]
    pub fn read(&self, addr: usize) -> u8 {
        match self.memory.read(addr) {
            Ok(v) => v,
            Err(_) => 0x00, // Failed read
        }
    }

    #[inline]
    pub fn write(&mut self, addr: usize, value: u8) {
        self.memory.write(addr, value);
    }

    // Allows larger access
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
