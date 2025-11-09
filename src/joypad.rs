use bitflags::bitflags;
use serde::{Serialize, Deserialize}; // Import

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

// --- ADD THIS STRUCT ---
#[derive(Serialize, Deserialize)]
pub struct JoypadState {
    strobe: bool,
    button_index: u8,
    button_status: u8, // Store the raw bits
}
// --- END STRUCT ---

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
            return 0x41;
        }
        
        let response = (self.button_status.bits() >> self.button_index) & 1;

        if !self.strobe {
            self.button_index += 1;
        }
        0x40 | response
    }

    pub fn peek(&self) -> u8 {
        if self.button_index > 7 {
            return 1;
        }
        (self.button_status.bits() >> self.button_index) & 1
    }

    // --- ADD THESE METHODS ---
    pub fn save_state(&self) -> JoypadState {
        JoypadState {
            strobe: self.strobe,
            button_index: self.button_index,
            button_status: self.button_status.bits(),
        }
    }

    pub fn load_state(&mut self, state: &JoypadState) {
        self.strobe = state.strobe;
        self.button_index = state.button_index;
        self.button_status = JoypadButton::from_bits_truncate(state.button_status);
    }
    // --- END METHODS ---
}