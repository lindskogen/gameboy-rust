use bit_field::BitField;
use crate::dmg::traits::Tick;

pub struct VolumeEnvelope {
    timer: u8,
    period: u8,
    add_mode: bool,
    starting_volume: u8,
    volume: u8,
    finished: bool,
}

impl VolumeEnvelope {
    pub fn set_nr12(&mut self, v: u8) {
        self.starting_volume = v >> 4;
        self.add_mode = v.get_bit(3);
        self.period = v & 0b111;
    }

    pub fn get_nr12(&self) -> u8 {
        (self.starting_volume << 4) | if self.add_mode { 0b100 } else { 0 } | self.period
    }
    pub fn get_volume(&self) -> u8 {
        if self.period > 0 {
            self.volume
        } else {
            self.starting_volume
        }
    }

    pub fn trigger(&mut self) {
        self.volume = self.starting_volume;
        self.finished = false;
        self.timer = if self.period != 0 { self.period } else { 8 };
    }

    pub fn power_off(&mut self) {
        self.finished = false;
        self.timer = 0;

        self.starting_volume = 0;
        self.add_mode = false;
        self.period = 0;

        self.volume = 0;
    }
}

impl Default for VolumeEnvelope {
    fn default() -> Self {
        Self {
            timer: 0,
            period: 0,
            starting_volume: 0,
            volume: 0,
            finished: true,
            add_mode: false,
        }
    }
}


impl Tick for VolumeEnvelope {
    fn tick(&mut self) {
        if self.finished {
            return;
        }

        self.timer = self.timer.saturating_sub(1);

        if self.timer == 0 {
            self.timer = if self.period != 0 { self.period } else { 8 };

            if self.add_mode && self.volume < 15 {
                self.volume += 1;
            }
            if !self.add_mode && self.volume > 0 {
                self.volume -= 1;
            }
            if self.volume == 0 || self.volume == 15 {
                self.finished = true;
            }
        }
    }
}
