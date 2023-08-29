use bit_field::BitField;
use crate::dmg::sound::common::ChannelCommon;
use crate::dmg::sound::traits::{Mem, Tick};
use crate::dmg::sound::volume_envelope::VolumeEnvelope;

pub struct Channel4 {
    pub common: ChannelCommon,

    timer: u8,
    clock_shift: u8,
    width_mode: bool,
    divisor_code: u8,
    lfsr: u16,
    divisors: [u8; 8],

    pub volume_envelope: VolumeEnvelope,
}


impl Default for Channel4 {
    fn default() -> Self {
        Self {
            common: ChannelCommon::new(4),
            timer: 0,
            clock_shift: 0,
            width_mode: false,
            divisor_code: 0,
            lfsr: 0x7fff,
            divisors: [8, 16, 32, 48, 64, 80, 96, 112],
            volume_envelope: VolumeEnvelope::default(),
        }
    }
}

impl Mem for Channel4 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff1f => 0xff,
            0xff20 => 0xff,
            0xff21 => self.volume_envelope.get_nr12(),
            0xff22 => {
                self.clock_shift << 4 | if self.width_mode { 0x08 } else { 0 } | self.divisor_code
            }
            0xff23 => {
                0xbf | if self.common.length_counter.is_enabled() {
                    0x40
                } else {
                    0
                }
            }
            _ => unreachable!("C4: Unmapped read")
        }
    }

    fn write(&mut self, addr: u16, v: u8) {
        match addr {
            0xff1f => {}
            0xff20 => {
                self.common.length_counter.set_length(v & 0x3f)
            }
            0xff21 => {
                self.common.dac_enabled = (v & 0xf8) != 0;
                self.common.ch_enabled = self.common.is_channel_enabled();
                self.volume_envelope.set_nr12(v);
            }
            0xff22 => {
                self.clock_shift = v >> 4;
                self.width_mode = v.get_bit(3);
                self.divisor_code = v & 0b111;
            }
            0xff23 => {
                self.common.length_counter.set_nr14(v);

                if self.common.length_counter.is_enabled() && self.common.length_counter.is_zero() {
                    self.common.ch_enabled = false;
                } else if v.get_bit(7) {
                    self.trigger()
                }
            },
            _ => unreachable!("CH4: Unmapped write")
        }
    }
}


impl Tick for Channel4 {
    fn tick(&mut self) {
        self.timer = self.timer.saturating_sub(1);

        if self.timer == 0 {
            self.timer = self.divisors[self.divisor_code as usize] << self.clock_shift;

            let result = ((self.lfsr & 1) ^ ((self.lfsr >> 1) & 1)) != 0;
            self.lfsr >>= 1;
            self.lfsr |= if result { 1 << 14 } else { 0 };

            if self.width_mode {
                self.lfsr &= !0x40;
                self.lfsr |= if result { 0x40 } else { 0 };
            }

            self.common.output = if self.common.is_channel_enabled() && !self.lfsr.get_bit(0) {
                self.volume_envelope.get_volume()
            } else {
                0
            };
        }
    }
}

impl Channel4 {
    fn trigger(&mut self) {
        self.volume_envelope.trigger();
        self.timer = self.divisors[self.divisor_code as usize] << self.clock_shift;

        self.lfsr = 0x7fff;

        self.common.ch_enabled = self.common.dac_enabled;
    }
    pub fn power_off(&mut self) {

        self.volume_envelope.power_off();
        self.common.length_counter.power_off();

        self.common.ch_enabled = false;
        self.common.dac_enabled = false;

        self.clock_shift = 0;
        self.width_mode = false;
        self.divisor_code = 0;
    }
}
