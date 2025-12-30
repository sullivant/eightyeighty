use crate::memory::{self, Memory};


// For mapping I/O devices
pub trait IoDevice {
    // Standard generic 
    fn input(&mut self, port: u8) -> u8;
    fn output(&mut self, port: u8, value: u8);

    // Direct bit control
    fn set_port(&mut self, port: u8, value: u8);
    fn set_bit(&mut self, port: u8, bit: u8);
    fn clear_bit(&mut self, port: u8, bit: u8);
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
    
    fn set_port(&mut self, _port: u8, _value: u8) {
        println!("Setting port in the null device.  Not cool.");
    }
    fn set_bit(&mut self, _port: u8, _bit: u8) {
        println!("Setting bit in the null device.  Not cool.");
    }
    fn clear_bit(&mut self, _port: u8, _bit: u8) {
        println!("Clearing bit in the null device.  Not cool.");
    }
}

pub struct Bus {
    memory: Memory,
    pub io: Box<dyn IoDevice>,

    pending_interrupt: Option<u8>, // Basically to hold RST 0-7
}

impl Bus {
    // Initial bus creation has no mapped IO device
    #[must_use]
    pub fn new(memory: Memory) -> Self{
        Self {
            memory,
            io: Box::new(NullDevice), // No real device to start
            pending_interrupt: None,
        }
    }

    // Create a bus with an IO device if wanted
    #[must_use]
    pub fn with_io(memory: Memory, io: Box<dyn IoDevice>) -> Self {
        Self { memory, io, pending_interrupt: None }
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
        println!("in bus.rs:input");
        self.print_io_ptr();
        self.io.input(port)
    }

    pub fn print_io_ptr(&self) {
        // Get a raw pointer to the trait object inside the Box
        let raw_ptr = &*self.io as *const dyn IoDevice;

        println!("Bus.io points to trait object at: {:p}", raw_ptr);

        // Get the raw pointer from the fat pointer:
        let (data_ptr, _vtable_ptr): (*const (), *const ()) = unsafe { 
            std::mem::transmute(raw_ptr)
        };

        println!("Bus.io data pointer (concrete object) is at: {:p}", data_ptr);
    }


    #[inline]
    pub fn output(&mut self, port: u8, value: u8) {
        self.io.output(port, value);
    }

    // Interrupts

    /// Stores the interrupt in the pending position
    pub fn request_interrupt(&mut self, rst: u8) {
        if rst > 7 { return; } // Only allowing 0-7
        self.pending_interrupt = Some(rst);
    }

    /// Takes the interrupt from the pending position
    pub fn take_interrupt(&mut self) -> Option<u8> {
        let i = self.pending_interrupt;
        self.pending_interrupt = None;
        i
    }

    /// Simply shows the interrupt but does not take
    pub fn peek_interrupt(&self) -> Option<u8> {
        self.pending_interrupt
    }
}
