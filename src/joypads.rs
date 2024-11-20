use bitflags::bitflags;

bitflags! {
    // https://wiki.nesdev.com/w/index.php/Controller_reading_code
    pub struct JoypadButton: u8 {
        const RIGHT             = 0b10000000;
        const LEFT              = 0b01000000;
        const DOWN              = 0b00100000;
        const UP                = 0b00010000;
        const START             = 0b00001000;
        const SELECT            = 0b00000100;
        const BUTTON_B          = 0b00000010;
        const BUTTON_A          = 0b00000001;
    }
}

pub struct Joypad {
    strobe: bool,     // is it in read mode or write mode
    button_index: u8, // pointer to a button
    pub button_status: JoypadButton,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            strobe: false,
            button_index: 0,
            button_status: JoypadButton::from_bits_truncate(0),
        }
    }

    pub fn write(&mut self, data: u8) {
        //println!("Current Status: {}", self.button_status.bits);
        //println!("Button Status in Write: {}", self.button_status.bits);
        self.strobe = data & 1 == 1; // set strobe mode to on if bit 1 of data is set
        //println!("Joypad strobe mode: {}", self.strobe);
        if self.strobe { // if it is to be on,
            self.button_index = 0 // initialise the button pointer for reads.
        }
    }

    pub fn read(&mut self) -> u8 {
        //println!("Button Pointer: {}", self.button_index);
        if self.button_index > 7 { // if button pointer exceeds, a read on an NES will always keep returning 1
            //println!("Joypad read: Button {} state: {} given button status {}", self.button_index, 1, self.button_status.bits);
            //println!("Testing");
            return 1;
        }

        // otherwise...
        let response = (self.button_status.bits & (1 << self.button_index)) >> self.button_index;
        //println!("Joypad read: Button {} state: {} given button status {}", self.button_index, response, self.button_status.bits);

        // self.button_status.bits & (1 << self.button_index) isolates the bit corresponding to the current button 
        // (A = index 0, B = index 1, and so on) from button_status.bits.
        // (1 << self.button_index) creates a mask with a 1 in the position of self.button_index: 
        
        // Masking is basically isolating the only bit you want to observe using the left shift bitwise operation
        // with the button pointer.

        // When self.button_status.bits is ANDed with this mask, only the bit for the current button state remains.
        
        // Shifting this result right by self.button_index normalizes it to 0 or 1, which becomes the final response 
        // (the current button state).

        if !self.strobe && self.button_index <= 7 { // if strobe mode is off and button pointer is in a valid read range
            self.button_index += 1; // increment the button pointer
        }
        //println!("Response: {} for button index: {}", response, self.button_index);
        response // and return the response
    }

    pub fn set_button_pressed_status(&mut self, button: JoypadButton, pressed: bool) {
        self.button_status.set(button, pressed);
        //println!("Button {:?} with status {} is now {}", button, self.button_status.bits, pressed);
    }

    /*pub fn select_joypad(&mut self, data: u8) -> u8 {
        
    }*/
}
