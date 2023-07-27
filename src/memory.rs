// use std::fmt;

use std::fmt;

use crate::constants::RAM_SIZE;

/// Memory
///
/// TODO: Make this able to output a section of data by slice, for processing by the
/// memory display window.

const SLICE_SIZE: usize = 16;

// Let's see how long we can last as full private?
#[derive(Clone)]
pub struct Memory {
    data: [u8; RAM_SIZE],
    pub table_start: usize, // If set, will allow fmt::Display to be truncated/walked
    pub table_stop: usize,  // If set, will allow fmt::Display to be truncated/walked
}

impl fmt::Display for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let iter = &mut self.data[self.table_start..=self.table_stop].chunks(SLICE_SIZE);
        let mut idx: usize = self.table_start;

        let mut out = format!("{}\n", table_header());

        for element in iter {
            out = format!("{out}{idx:04X} {element:02X?}\n");
            idx += SLICE_SIZE;
        }

        write!(f, "{out}")?;
        Ok(())
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            data: [0; RAM_SIZE],
            table_start: 0,
            table_stop: RAM_SIZE - 1,
        }
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
