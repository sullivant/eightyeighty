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
