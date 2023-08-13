use bitflags::bitflags;
use serde::{Serialize, Deserialize};

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct InterruptFlag: u8 {
        const V_BLANK  = 1 << 0;
        const LCD_STAT = 1 << 1;
        const TIMER    = 1 << 2;
        const SERIAL   = 1 << 3;
        const JOYPAD   = 1 << 4;
    }
}

impl InterruptFlag {
    pub fn highest_prio_bit(&self) -> InterruptFlag {
        if self.contains(InterruptFlag::V_BLANK) {
            InterruptFlag::V_BLANK
        } else if self.contains(InterruptFlag::LCD_STAT) {
            InterruptFlag::LCD_STAT
        } else if self.contains(InterruptFlag::TIMER) {
            InterruptFlag::TIMER
        } else if self.contains(InterruptFlag::SERIAL) {
            InterruptFlag::SERIAL
        } else if self.contains(InterruptFlag::JOYPAD) {
            InterruptFlag::JOYPAD
        } else {
            InterruptFlag::empty()
        }
    }
    pub fn interrupt_starting_address(&self) -> Option<u16> {
        match self.highest_prio_bit() {
            InterruptFlag::V_BLANK => Some(0x40),
            InterruptFlag::LCD_STAT => Some(0x48),
            InterruptFlag::TIMER => Some(0x50),
            InterruptFlag::SERIAL => Some(0x58),
            InterruptFlag::JOYPAD => Some(0x60),
            _ => None,
        }
    }
}
