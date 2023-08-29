use bit_field::BitField;

use crate::dmg::sound::common::ChannelCommon;
use crate::dmg::sound::traits::{Mem, Tick};

pub struct Channel3 {
    pub common: ChannelCommon,
    timer: u32,
    position: usize,
    ticks_since_read: u32,
    frequency: u32,
    volume_code: u8,
    last_address: usize,
    wave_table: [u8; 16],
}


impl Default for Channel3 {
    fn default() -> Self {
        Self {
            common: ChannelCommon::new(3),
            timer: 0,
            position: 4,
            ticks_since_read: 0,
            frequency: 0,
            volume_code: 0,
            last_address: 0,
            wave_table: [0; 16],
        }
    }
}

impl Mem for Channel3 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff30..=0xff3f => {
                if self.common.is_channel_enabled() {
                    if self.ticks_since_read < 2 {
                        self.wave_table[self.last_address]
                    } else {
                        0xff
                    }
                } else {
                    self.wave_table[addr as usize - 0xff30]
                }
            }
            0xff1a => {
                0x7f | if self.common.dac_enabled { 0x80 } else { 0 }
            }
            0xff1b => 0xff,
            0xff1c => {
                self.volume_code << 5 | 0x9f
            }
            0xff1d => 0xff,
            0xff1e => {
                0xbf | if self.common.length_counter.is_enabled() { 0x40 } else { 0 }
            }
            _ => unreachable!("CH3: Unmapped read")
        }
    }

    fn write(&mut self, addr: u16, v: u8) {
        match addr {
            0xff30..=0xff3f => {
                if self.common.is_channel_enabled() {
                    if self.ticks_since_read < 2 {
                        self.wave_table[self.last_address] = v;
                    }
                } else {
                    self.wave_table[addr as usize - 0xff30] = v;
                }
            },
            0xff1a => {
                self.common.dac_enabled = v.get_bit(7);
                self.common.ch_enabled = self.common.is_channel_enabled();
            },
            0xff1b => {
                self.common.length_counter.set_length(v);
            },
            0xff1c => {
                self.volume_code = (v >> 5) & 0b11;
            },
            0xff1d => {
                self.frequency = (self.frequency & 0x700) | v as u32;
            },
            0xff1e => {
                self.common.length_counter.set_nr14(v);
                self.frequency = (self.frequency & 0xff) | ((v as u32 & 0b111) << 8);

                if self.common.length_counter.is_enabled() && self.common.length_counter.is_zero() {
                    self.common.ch_enabled = false;
                } else if v.get_bit(7) {
                    self.trigger()
                }
            },
            _ => unreachable!("CH3: Unmapped write")
        }
    }
}


impl Tick for Channel3 {
    fn tick(&mut self) {
        self.ticks_since_read += 1;

        self.timer = self.timer.saturating_sub(1);

        if self.timer == 0 {
            self.timer = (2048 - self.frequency) << 1;

            if self.common.is_channel_enabled() {
                self.ticks_since_read = 0;
                self.last_address = self.position >> 1;
                self.common.output = self.wave_table[self.last_address];

                if self.position.get_bit(0) {
                    self.common.output &= 0x0f;
                } else {
                    self.common.output >>= 4;
                }

                if self.volume_code > 0 {
                    self.common.output >>= self.volume_code - 1;
                } else {
                    self.common.output = 0;
                }

                self.position = (self.position + 1) & 31;
            } else {
                self.common.output = 0;
            }
        }
    }
}

impl Channel3 {
    fn trigger(&mut self) {
        if self.common.is_channel_enabled() && self.timer == 2 {
            let mut pos = self.position >> 1;

            if pos < 4 {
                self.wave_table[0] = self.wave_table[pos];
            } else {
                pos &= !0b11;

                self.wave_table.copy_within(pos..pos + 4, 0);
            }
        }

        self.timer = 6; // Note: You need this to get blargg's "09-wave read while on" test

        self.position = 0;
        self.last_address = 0;
        self.common.ch_enabled = self.common.dac_enabled;
    }

    pub fn power_off(&mut self) {
        self.common.length_counter.power_off();

        self.common.ch_enabled = false;
        self.common.dac_enabled = false;

        self.position = 0;
        self.frequency = 0;
        self.volume_code = 0;

        self.ticks_since_read = 0;
        self.last_address = 0;
    }
}
