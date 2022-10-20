use crate::constants::RAM_SIZE;

// Let's see how long we can last as full private?
#[derive(Clone)]
pub struct Memory {
    data: [u8; RAM_SIZE],
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
