use crate::dmg::sound::length_counter::LengthCounter;
use crate::dmg::sound::traits::Tick;

pub struct ChannelCommon {
    pub channel_no: u8,
    pub channel_enabled: bool,
    pub dac_enabled: bool,
    pub output: u8,
    pub length_counter: LengthCounter,
}


impl ChannelCommon {
    pub fn new(channel_no: u8) -> Self {
        Self {
            dac_enabled: false,
            channel_no,
            output: 0,
            channel_enabled: false,
            length_counter: LengthCounter::default()
        }
    }

    pub fn tick_channel_length(&mut self) {
        self.length_counter.tick();
        if self.length_counter.is_enabled() && self.length_counter.is_zero() {
            self.channel_enabled = false;
        }
    }
}


pub const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 0],
];