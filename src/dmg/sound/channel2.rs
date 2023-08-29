use crate::dmg::sound::common::{ChannelCommon, DUTY_TABLE};
use crate::dmg::sound::traits::{Mem, Tick};
use crate::dmg::sound::volume_envelope::VolumeEnvelope;

pub struct Channel2 {
    pub common: ChannelCommon,
    duty: usize,
    timer: u32,
    sequence: usize,
    frequency: u32,

    pub volume_envelope: VolumeEnvelope,
}


impl Default for Channel2 {
    fn default() -> Self {
        Self {
            common: ChannelCommon::new(2),
            timer: 0,
            sequence: 0,
            duty: 0,
            frequency: 0,
            volume_envelope: VolumeEnvelope::default(),
        }
    }
}

impl Mem for Channel2 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            _ => 0xff
        }
    }

    fn write(&mut self, addr: u16, v: u8) {}
}


impl Tick for Channel2 {
    fn tick(&mut self) {
        self.timer = self.timer.saturating_sub(1);

        if self.timer == 0 {
            self.timer = (2048 - self.frequency) << 2;
            self.sequence = (self.sequence + 1) % 7;

            self.common.output = if self.common.channel_enabled {
                if DUTY_TABLE[self.duty][self.sequence] == 1 {
                    self.volume_envelope.get_volume()
                } else {
                    0
                }
            } else {
                0
            };
        }
    }
}
