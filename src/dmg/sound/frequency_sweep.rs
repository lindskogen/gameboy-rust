use bit_field::BitField;
use crate::dmg::sound::traits::Tick;

pub struct FrequencySweep {
    enabled: bool,
    overflow: bool,
    has_negated: bool,
    timer: u8,
    frequency: u16,
    shadow_frequency: u16,
    period: u8,
    negate: bool,
    shift: u8,
}

impl FrequencySweep {
    fn calculate(&mut self) -> u16 {
        let mut new_freq = self.shadow_frequency >> self.shift;

        if self.negate {
            new_freq = self.shadow_frequency - new_freq;
            self.has_negated = true;
        } else {
            new_freq = self.shadow_frequency + new_freq;
        }

        if new_freq > 2047 {
            self.overflow = true;
        }

        new_freq
    }

    pub fn get_frequency(&self) -> u32 {
        self.frequency as u32
    }

    pub fn get_nr10(&self) -> u8 {
        0x80 | (self.period << 4) | (if self.negate { 0b100 } else { 0 }) | self.shift
    }
    pub fn set_nr10(&mut self, v: u8) {
        self.period = (v >> 4) & 0b111;
        self.negate = v.get_bit(3);
        self.shift = v & 0b111;

        if self.has_negated && !self.negate {
            self.overflow = true;
        }
    }

    pub fn is_enabled(&self) -> bool {
        !self.overflow
    }

    pub fn set_nr13(&mut self, v: u8) {
        self.frequency = (self.frequency & 0x700) | v as u16;
    }

    pub fn set_nr14(&mut self, v: u8) {
        self.frequency = (self.frequency & 0xff) | ((v as u16 & 0b111) << 8)
    }

    pub fn trigger(&mut self) {
        self.overflow = false;
        self.has_negated = false;
        self.shadow_frequency = self.frequency;
        self.timer = if self.period != 0 { self.period } else { 8 };
        self.enabled = self.period != 0 || self.shift != 0;

        if self.shift > 0 {
            self.calculate();
        }
    }

    pub fn power_off(&mut self) {
        self.enabled = false;
        self.overflow = false;
        self.has_negated = false;

        self.timer = 0;

        self.frequency = 0;
        self.shadow_frequency = 0;

        self.period = 0;
        self.negate = false;
        self.shift = 0;
    }
}


impl Default for FrequencySweep {
    fn default() -> Self {
        Self {
            negate: false,
            shadow_frequency: 0,
            has_negated: false,
            enabled: false,
            period: 0,
            shift: 0,
            timer: 0,
            overflow: false,
            frequency: 0,
        }
    }
}


impl Tick for FrequencySweep {
    fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        self.timer = self.timer.saturating_sub(1);

        if self.timer == 0 {
            self.timer = if self.period != 0 { self.period } else { 8 };

            if self.period != 0 {
                let new_freq = self.calculate();

                if !self.overflow && self.shift != 0 {
                    self.shadow_frequency = new_freq;
                    self.frequency = new_freq;

                    self.calculate();
                }
            }
        }
    }
}
