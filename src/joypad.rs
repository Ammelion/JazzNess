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
    strobe: bool,     
    button_index: u8,  
    button_status: JoypadButton,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            strobe: false,
            button_index: 0,
            button_status: JoypadButton::empty(),
        }
    }

    pub fn set_button_pressed_status(&mut self, button: JoypadButton, pressed: bool) {
        self.button_status.set(button, pressed);
    }

    pub fn write(&mut self, data: u8) {
        self.strobe = data & 1 == 1;
        if self.strobe {
            self.button_index = 0;
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.button_index > 7 {
            // After 8 bits, return 0x41 to identify as a standard controller
            // This is crucial for passing the hardware check in many games.
            return 0x41; // <<< FIX #1
        }
        
        // Read the current button's state based on the index
        let response = (self.button_status.bits() >> self.button_index) & 1;

        // Increment the index ONLY if strobe mode is off
        if !self.strobe {
            self.button_index += 1;
        }
        
        // Return 0x40 plus the button bit. This mimics open bus behavior
        // combined with the controller's data line. While games often
        // ignore the upper bits, returning 0x40 | response is most accurate.
        0x40 | response // <<< FIX #2
    }

    pub fn peek(&self) -> u8 {
        if self.button_index > 7 {
            return 1;
        }
        (self.button_status.bits() >> self.button_index) & 1
    }

}
