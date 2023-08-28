use crate::dmg::sound::traits::Tick;

pub struct LengthCounter {
    enabled: bool,
    length: u8,
    full_length: u8,
    pub frame_sequencer: u8
}

impl Default for LengthCounter {
    fn default() -> Self {
        Self {
            length: 0,
            full_length: 64,
            enabled: false,
            frame_sequencer: 0
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
