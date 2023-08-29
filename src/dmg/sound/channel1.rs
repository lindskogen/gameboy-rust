use bit_field::BitField;
use crate::dmg::sound::common::{ChannelCommon, DUTY_TABLE};
use crate::dmg::sound::frequency_sweep::FrequencySweep;
use crate::dmg::sound::traits::{Mem, Tick};
use crate::dmg::sound::volume_envelope::VolumeEnvelope;


pub struct Channel1 {
    pub common: ChannelCommon,
    duty: u8,
    timer: u32,
    sequence: usize,

    pub volume_envelope: VolumeEnvelope,
    pub frequency_sweep: FrequencySweep,
}

impl Default for Channel1 {
    fn default() -> Self {
        Self {
            common: ChannelCommon::new(1),
            duty: 0,
            timer: 0,
            sequence: 0,
            volume_envelope: VolumeEnvelope::default(),
            frequency_sweep: FrequencySweep::default(),
        }
    }
}

impl Mem for Channel1 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff10 => self.frequency_sweep.get_nr10(),
            0xff11 => (self.duty << 6) | 0x3f,
            0xff12 => self.volume_envelope.get_nr12(),
            0xff13 => 0xff,
            0xff14 => {
                0xbf | if self.common.length_counter.enabled {
                    0x40
                } else { 0 }
            }
            _ => unreachable!("CH1: Unmapped read")
        }
    }

    fn write(&mut self, addr: u16, v: u8) {
        println!("write {:4X} {:8b}", addr, v);
        match addr {
            0xff10 => {
                self.frequency_sweep.set_nr10(v);

                if !self.frequency_sweep.is_enabled() {
                    self.common.channel_enabled = false;
                }
            }
            0xff11 => {
                self.duty = v >> 6;
                self.common.length_counter.set_length(v & 0x3f)
            }
            0xff12 => {
                self.common.dac_enabled = (v & 0xf8) != 0;
                self.common.channel_enabled = self.common.channel_enabled && self.common.dac_enabled;
                self.volume_envelope.set_nr12(v);
            }
            0xff13 => {
                self.frequency_sweep.set_nr13(v);
            }
            0xff14 => {
                self.frequency_sweep.set_nr14(v);
                self.common.length_counter.set_nr14(v);

                if self.common.length_counter.enabled && self.common.length_counter.is_zero() {
                    self.common.channel_enabled = false;
                } else if v.get_bit(7) {
                    self.trigger()
                }
            }
            _ => unreachable!("CH1: Unmapped write")
        }
    }
}


impl Tick for Channel1 {
    fn tick(&mut self) {
        self.timer = self.timer.saturating_sub(1);

        if self.timer == 0 {
            self.timer = (2048 - self.frequency_sweep.get_frequency()) << 2;
            self.sequence = (self.sequence + 1) % 7;

            println!("ch en {}", self.common.channel_enabled);

            self.common.output = if self.common.channel_enabled && self.common.dac_enabled {
                if DUTY_TABLE[self.duty as usize][self.sequence] == 1 {
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


impl Channel1 {
    fn trigger(&mut self) {
        self.timer = (2048 - self.frequency_sweep.get_frequency()) << 2;
        self.volume_envelope.trigger();
        self.frequency_sweep.trigger();
        if self.frequency_sweep.is_enabled() {
            self.common.channel_enabled = self.common.dac_enabled;
        } else {
            self.common.channel_enabled = false;
        }
    }

    pub fn tick_frequency_sweep(&mut self) {
        self.frequency_sweep.tick();

        if !self.frequency_sweep.is_enabled() {
            self.common.channel_enabled = false;
        }
    }
}
