// use std::fmt;

use std::fmt;
use serde::{Deserialize,Serialize};
use serde_big_array::BigArray;

use crate::constants::{RAM_SIZE, VRAM_END, VRAM_START};

/// Memory
///
/// TODO: Make this able to output a section of data by slice, for processing by the
/// memory display window.

const SLICE_SIZE: usize = 16;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Memory {
    #[serde(with = "BigArray")]
    data: [u8; RAM_SIZE]
}

/// When returning (for display) the memory, we need to represent this as a JSON string
/// so that the caller can parse it in the way necessary.
/// 
/// TODO: Consider a separate "as JSON" function, to allow for display of ram in other ways.
impl fmt::Display for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Serialize it to a JSON string.
        let j = serde_json::to_string(self).unwrap();
        write!(f, "{j}")?;
        Ok(())
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory {
    pub const fn new() -> Memory {
        Memory {
            data: [0; RAM_SIZE]
        }
    }

    /// Returns a slice (16) of ram beginning at the provided starting point
    pub fn get_slice(&self, start: usize) -> [u8; 16] {
        let mut ret: [u8; 16] = [0; 16];

        for n in 0..16 {
            ret[n] = self.read(n + start).unwrap_or_default();
        }

        ret
    }

    /// Returns the section of memory dedicated to Video.
    pub fn get_vram(&self) -> &[u8] {
        // let mut ret: [u8; VRAM_SIZE] = [0; VRAM_SIZE];

        &self.data[VRAM_START..=VRAM_END]
    }

    // Returns a cloned copy of the value in memory, or an error if unable to read
    // from that portion.
    pub fn read(&self, loc: usize) -> Result<u8, String> {
        match self.data.get(loc) {
            Some(v) => Ok(*v),
            None => Err(format!("RAM: Unable to read at location: {loc:#04X}")),
        }
    }

    // Writes to a location in memory
    pub fn write(&mut self, loc: usize, val: u8) {
        if loc > RAM_SIZE - 1 {
            return
        }

        self.data[loc] = val;
    }

    pub fn get_memory_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn get_memory_size(&self) -> usize {
        self.data.len()
    }
    
}

#[cfg(test)]
mod tests {
    use crate::constants::RAM_SIZE;

    use super::Memory;

    #[test]
    fn test_new() {
        let mem = Memory::new();
        let array: [u8; RAM_SIZE] = [0; RAM_SIZE];

        assert_eq!(mem.data, array);
    }

    #[test]
    fn test_read() {
        let mem: Memory = Memory::new();
        assert_eq!(mem.read(0x00).unwrap(), 0x00);

        assert_eq!(
            mem.read(RAM_SIZE),
            Err(format!("RAM: Unable to read at location: {RAM_SIZE:#04X}"))
        );
    }
}
