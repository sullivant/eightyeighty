/// This module contains common, simple, and easily documentable
/// functions to be used, possibly, in various locations

// Makes a memory pointer by simply concatenating the two values
#[must_use]
pub fn make_pointer(dl: u8, dh: u8) -> usize {
    usize::from(u16::from(dh) << 8 | u16::from(dl))
}

// If number of ones in a number's binary representation is even,
// parity flag is TRUE (1) else it is FALSE (0)
#[must_use]
pub fn get_parity(v: u16) -> bool {
    v.count_ones() % 2 == 0
}

// Returns true if MSB = 1
#[must_use]
pub fn get_sign(x: u8) -> bool {
    (0b1000_0000 & x) != 0
}

// Returns true if an addition will case an aux carry
// value: the value we are trying to add to source
// source: the source that value is added to
#[must_use]
pub fn will_ac(value: u8, source: u8) -> bool {
    ((value & 0x0F) + (source & 0x0F)) & 0x10 == 0x10
}
