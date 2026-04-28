pub struct Settings { 
    pub refresh_ms: u64, 
    pub battery_mah: u32, 
    pub selected: usize, 
    pub focused: bool 
}

impl Default for Settings {
    fn default() -> Self {
        Self { 
            refresh_ms: 500, 
            battery_mah: 4000, 
            selected: 0, 
            focused: false 
        }
    }
}