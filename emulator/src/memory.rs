// use std::fmt;

use std::fmt;
use serde::{Deserialize,Serialize};
use serde_big_array::BigArray;

use crate::constants::RAM_SIZE;

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

    pub fn get_slice(&self, start: usize) -> [u8; 16] {
        let mut ret: [u8; 16] = [0; 16];

        for n in 0..16 {
            ret[n] = match self.read(n + start) {
                Ok(x) => x,
                Err(_) => 0
            };
        }

        return ret;
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
    // TODO: Make this respect things a little more, maybe write via range instead?
    pub fn write(&mut self, loc: usize, val: u8) -> Result<(), String> {
        if loc > RAM_SIZE - 1 {
            return Err(format!("Unable to write to memory location: {loc:04X}"));
        }

        self.data[loc] = val;

        Ok(())
    }
}

// Creates a simple table header used in displaying ram contents.
pub fn table_header() -> String {
    let mut header = [0; 16];
    for (i, item) in header.iter_mut().enumerate() {
        *item = i;
    }
    format!("0000 {header:02X?}")
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
