pub struct ScrollRegister {
    pub scroll_x: u8,
    pub scroll_y: u8,
    pub scroll_switch: bool,
}

impl ScrollRegister {
    pub fn new() -> Self {
        ScrollRegister {
            scroll_x: 0,
            scroll_y: 0,
            scroll_switch: false,
        }
    }

    pub fn write(&mut self, data: u8) {
        if !self.scroll_switch {
            self.scroll_x = data;
        } else {
            self.scroll_y = data;
        }
        self.scroll_switch = !self.scroll_switch;
    }

    pub fn reset_scroll_switch(&mut self) {
        self.scroll_switch = false;
    }
}