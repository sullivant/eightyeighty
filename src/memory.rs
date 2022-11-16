// use std::fmt;

use crate::constants::RAM_SIZE;
use tabled::{TableIteratorExt, Extract};
use tabled::{Table, Style};

/// Memory
///
/// TODO: Make this able to output a section of data by slice, for processing by the
/// memory display window.

// Let's see how long we can last as full private?
#[derive(Clone)]
pub struct Memory {
    data: [u8; RAM_SIZE],
}

// impl fmt::Display for Memory {
    // fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    //     for (i,v) in self.data[0x00..=0x1F00].iter().enumerate() {
    //         if i == 0 {
    //             write!(f,"XXXX : 00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F\n{:0>4X} : ",i)?;
    //         }
    //         if i > 1 && i % 16 == 0 { write!(f,"|\n{:0>4X} : ",i+1)?}

    //         write!(f,"{:0>2X} ",v)?;
    //     }

    //     Ok(())
    // }
// }

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            data: [0; RAM_SIZE],
        }
    }

    // Returns a cloned copy of the value in memory, or an error if unable to read
    // from that portion.
    pub fn read(&self, loc: usize) -> Result<u8, String> {
        match self.data.get(loc) {
            Some(v) => Ok(*v),
            None => Err(format!("RAM: Unable to read at location: {:#04X}", loc)),
        }
    }

    // Writes to a location in memory
    // TODO: Make this respect things a little more, maybe write via range instead?
    pub fn write(&mut self, loc: usize, val: u8) -> Result<(), String> {
        if loc > RAM_SIZE - 1 {
            return Err(format!("Unable to write to memory location: {:04X}", loc));
        }

        self.data[loc] = val;

        Ok(())
    }

    // Pretty prints a table of the memory from start to (and inclusive of) end
    pub fn table(&mut self, start: usize, end: usize) {
        let numbers = [1, 2, 3];
        //self.data[0x00..=0xFF]
        let mut table = Table::new(&self.data);
        // println!("{}",table.with(Extract::segment(1..3, 1..)));

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
            Err(format!(
                "RAM: Unable to read at location: {:#04X}",
                RAM_SIZE
            ))
        );
    }
}
