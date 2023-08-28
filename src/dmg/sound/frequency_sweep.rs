use crate::dmg::sound::traits::Tick;

pub struct FrequencySweep {
    enabled: bool,
    overflow: bool,
    has_negated: bool,
    timer: u32,
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
}

impl FrequencySweep {
    pub fn get_frequency(&self) -> u32 {
        self.frequency as u32
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

        if self.timer == 0 {
            self.timer = if self.period != 0 { self.period } else { 8 } as u32;

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
