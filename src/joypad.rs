use bitflags::bitflags;

bitflags! {
    #[derive(Copy, Clone)]
    pub struct JoypadButton: u8 {
        const BUTTON_A          = 0b00000001;
        const BUTTON_B          = 0b00000010;
        const SELECT            = 0b00000100;
        const START             = 0b00001000;
        const UP                = 0b00010000;
        const DOWN              = 0b00100000;
        const LEFT              = 0b01000000;
        const RIGHT             = 0b10000000;
    }
}

pub struct Joypad {
    strobe: bool,      // True when the strobe bit is set (in write mode)
    button_index: u8,  // The current button being reported (0-7)
    button_status: JoypadButton, // The current pressed state of all buttons
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            strobe: false,
            button_index: 0,
            button_status: JoypadButton::empty(),
        }
    }

    // This is called when the host machine (your keyboard) presses or releases a key.
    pub fn set_button_pressed_status(&mut self, button: JoypadButton, pressed: bool) {
        self.button_status.set(button, pressed);
    }

    // This is called when the CPU writes to address 0x4016.
    pub fn write(&mut self, data: u8) {
        // Only the first bit of the write matters.
        self.strobe = data & 1 == 1;
        if self.strobe {
            // If strobe is on, we reset the button index to the beginning (Button A).
            self.button_index = 0;
        }
    }

    // This is called when the CPU reads from address 0x4016.
    pub fn read(&mut self) -> u8 {
        if self.button_index > 7 {
            // After all 8 buttons have been read, the controller returns 1.
            return 1;
        }
        
        // Check if the current button is pressed.
        let response = (self.button_status.bits() >> self.button_index) & 1;

        // If strobe is OFF, we advance to the next button for the next read.
        if !self.strobe {
            self.button_index += 1;
        }
        
        response
    }

    pub fn peek(&self) -> u8 {
        if self.button_index > 7 {
            return 1;
        }
        (self.button_status.bits() >> self.button_index) & 1
    }

}
