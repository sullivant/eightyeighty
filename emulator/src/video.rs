/// This holds any steps or features related to processing the video portion of memory.
/// This means this module needs access (and reference to) various portions of
/// the system at large, such as these at the bare minimum
///     The calling library (lib)
///     The memory
///     The interrupt subsystem
///
/// Memory is a "child" of CPU - but it really could be brought into the greater lib,
/// accessed by CPU (and video) via wrapper/channels.  I need to research that a little
/// bit more.  For now, I reckon, a thread spawned off to process video memory could have
/// passed to it a reference to a slice of memory based on `CPU.get_memory`(<slice>).
///
/// Interrupts are a yet to be developed or determined thing.  CPU needs to respond when
/// they are triggered, and they need to come from various channels.  This can likely be
/// done through a channel via the whole "Arc / Mutex" thing.  I don't know, but I do
/// know this is a great opportunity to read up on the idiomatic way to have inter-process
/// communication like that.  
///

pub struct Video {
    pub tick_count: usize,
}

impl Default for Video {
    fn default() -> Self {
        Self::new()
    }
}

impl Video {
    pub fn new() -> Video {
        Video { tick_count: 0 }
    }
}
