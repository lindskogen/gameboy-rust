use bit_field::BitField;
use crate::dmg::sound::common::{ChannelCommon, DUTY_TABLE};
use crate::dmg::traits::{Mem, Tick};
use crate::dmg::sound::volume_envelope::VolumeEnvelope;

pub struct Channel2 {
    pub common: ChannelCommon,
    duty: u8,
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
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xff15 => 0xff,
            0xff16 => {
                self.duty << 6 | 0x3f
            }
            0xff17 => {
                self.volume_envelope.get_nr12()
            }
            0xff18 => {
                0xff
            }
            0xff19 => {
                0xbf | if self.common.length_counter.is_enabled() {
                    0x40
                } else { 0 }
            }
            _ => unreachable!("CH1: Unmapped read")
        }
    }

    fn write_byte(&mut self, addr: u16, v: u8) {
        match addr {
            0xff15 => {}
            0xff16 => {
                self.duty = v >> 6;
                self.common.length_counter.set_length(v & 0x3f)
            }
            0xff17 => {
                self.common.dac_enabled = (v & 0xf8) != 0;
                self.common.ch_enabled = self.common.is_channel_enabled();
                self.volume_envelope.set_nr12(v);
            }
            0xff18 => {
                self.frequency = (self.frequency & 0x700) | v as u32;
            }
            0xff19 => {
                self.frequency = (self.frequency & 0xff) | (((v as u32) & 0b111) << 8);
                self.common.length_counter.set_nr14(v);

                if self.common.length_counter.is_enabled() && self.common.length_counter.is_zero() {
                    self.common.ch_enabled = false;
                } else if v.get_bit(7) {
                    self.trigger()
                }
            }
            _ => unreachable!("CH2: Unmapped write")
        }
    }
}


impl Tick for Channel2 {
    fn tick(&mut self) {
        self.timer = self.timer.saturating_sub(1);

        if self.timer == 0 {
            self.timer = (2048 - self.frequency) << 2;
            self.sequence = (self.sequence + 1) % 7;

            self.common.output = if self.common.ch_enabled {
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


impl Channel2 {
    fn trigger(&mut self) {
        self.timer = (2048 - self.frequency) << 2;
        self.volume_envelope.trigger();
        self.common.ch_enabled = self.common.dac_enabled;
    }

    pub fn power_off(&mut self) {
        self.volume_envelope.power_off();
        self.common.length_counter.power_off();

        self.common.ch_enabled = false;
        self.common.dac_enabled = false;

        self.sequence = 0;
        self.frequency = 0;

        self.duty = 0;
    }
}
