use bit_field::BitField;
use crate::dmg::sound::traits::Tick;

pub struct LengthCounter {
    pub enabled: bool,
    length: u8,
    full_length: u8,
    pub frame_sequencer: u8,
}

impl LengthCounter {
    pub fn is_zero(&self) -> bool {
        self.length == 0
    }
}

impl LengthCounter {
    pub fn set_nr14(&mut self, v: u8) {
        let enable = v.get_bit(6);
        let trigger = v.get_bit(7);

        if self.enabled {
            if trigger && self.is_zero() {
                self.length = if enable && self.frame_sequencer.get_bit(0) {
                    self.full_length - 1
                } else {
                    self.full_length
                };
            }
        } else if enable {
            if self.frame_sequencer.get_bit(0) {
                if self.length != 0 {
                    self.length -= 1;
                }

                if trigger && self.is_zero() {
                    self.length = self.full_length - 1;
                }
            }
        } else {
            if trigger && self.is_zero() {
                self.length = self.full_length;
            }
        }

        self.enabled = enable;
    }

    pub fn set_length(&mut self, v: u8) {
        if self.length == 0 {
            self.length = self.full_length
        } else {
            self.length = self.full_length - v;
        }
    }
}

impl Default for LengthCounter {
    fn default() -> Self {
        Self {
            length: 0,
            full_length: 64,
            enabled: false,
            frame_sequencer: 0,
        }
    }
}


impl Tick for LengthCounter {
    fn tick(&mut self) {
        if self.enabled && self.length > 0 {
            self.length -= 1;
        }
    }
}
