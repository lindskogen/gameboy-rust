use bit_field::BitField;
use bitflags::bitflags;
use crate::dmg::sound::channel1::Channel1;
use crate::dmg::sound::channel2::Channel2;
use crate::dmg::sound::channel3::Channel3;
use crate::dmg::sound::channel4::Channel4;

use crate::dmg::sound::traits::{Mem, Tick};

bitflags! {
    pub struct ChannelEnabled: u8 {
        const LEFT_1    = 1 << 7;
        const LEFT_2    = 1 << 6;
        const LEFT_3    = 1 << 5;
        const LEFT_4    = 1 << 4;
        const RIGHT_1   = 1 << 3;
        const RIGHT_2   = 1 << 2;
        const RIGHT_3   = 1 << 1;
        const RIGHT_4   = 1 << 0;
    }
}

pub struct Apu {
    master_volume: f32,
    enabled: bool,

    vin_left_enable: bool,
    vin_right_enable: bool,

    left_volume: u8,
    right_volume: u8,

    channel_enabled: ChannelEnabled,

    frequency_counter: u8,
    frame_sequencer_counter: u32,
    frame_sequencer: u8,

    channel1: Channel1,
    channel2: Channel2,
    channel3: Channel3,
    channel4: Channel4,
}

impl Apu {
    pub fn sample(&self) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;

        if self.channel_enabled.contains(ChannelEnabled::LEFT_1) {
            left += self.channel1.common.output as f32;
        }

        if self.channel_enabled.contains(ChannelEnabled::LEFT_2) {
            left += self.channel2.common.output as f32;
        }

        if self.channel_enabled.contains(ChannelEnabled::LEFT_3) {
            left += self.channel3.common.output as f32;
        }

        if self.channel_enabled.contains(ChannelEnabled::LEFT_4) {
            left += self.channel4.common.output as f32;
        }

        if self.channel_enabled.contains(ChannelEnabled::RIGHT_1) {
            right += self.channel1.common.output as f32;
        }

        if self.channel_enabled.contains(ChannelEnabled::RIGHT_2) {
            right += self.channel2.common.output as f32;
        }

        if self.channel_enabled.contains(ChannelEnabled::RIGHT_3) {
            right += self.channel3.common.output as f32;
        }

        if self.channel_enabled.contains(ChannelEnabled::RIGHT_4) {
            right += self.channel4.common.output as f32;
        }

        (
            (left / 16.0) * self.master_volume,
            (right / 16.0) * self.master_volume
        )
    }
}


impl Default for Apu {
    fn default() -> Self {
        Self {
            master_volume: 0.1,
            enabled: false,

            left_volume: 0,
            right_volume: 0,

            vin_left_enable: false,
            vin_right_enable: false,
            channel_enabled: ChannelEnabled::empty(),

            frame_sequencer: 0,
            frame_sequencer_counter: 8192,
            frequency_counter: 95,

            channel1: Channel1::default(),
            channel2: Channel2::default(),
            channel3: Channel3::default(),
            channel4: Channel4::default(),
        }
    }
}


impl Tick for Apu {
    fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        self.frame_sequencer_counter -= 1;

        if self.frame_sequencer_counter == 0 {
            self.frame_sequencer_counter = 8192;

            match self.frame_sequencer {
                0 => {
                    self.tick_all_channel_lengths();
                }
                1 => {
                    // noop
                }
                2 => {
                    self.channel1.frequency_sweep.tick();
                    self.tick_all_channel_lengths();
                }
                3 => {
                    // noop
                }
                4 => {
                    self.tick_all_channel_lengths();
                }
                5 => {
                    // noop
                }
                6 => {
                    self.channel1.tick_frequency_sweep();
                    self.tick_all_channel_lengths();
                }
                7 => {
                    self.channel1.volume_envelope.tick();
                    self.channel2.volume_envelope.tick();
                    // no volume envelope for ch 3
                    self.channel4.volume_envelope.tick();
                }
                _ => {}
            }

            self.frame_sequencer = (self.frame_sequencer + 1) & 7;

            self.channel1.common.length_counter.frame_sequencer = self.frame_sequencer;
            self.channel2.common.length_counter.frame_sequencer = self.frame_sequencer;

        }
        self.tick_all_channels();

        self.frequency_counter -= 1;

        if self.frequency_counter == 0 {
            self.frequency_counter = 95;
        }
    }
}


impl Apu {
    fn clear_all_registers(&mut self) {
        self.vin_left_enable = false;
        self.vin_right_enable = false;
        self.left_volume = 0;
        self.right_volume = 0;
        self.enabled = false;

        self.channel1.power_off();
        self.channel2.power_off();
        self.channel3.power_off();
        self.channel4.power_off();

        self.channel_enabled = ChannelEnabled::empty();
    }

    fn tick_all_channels(&mut self) {
        self.channel1.tick();
        self.channel2.tick();
        self.channel3.tick();
        self.channel4.tick();
    }

    fn tick_all_channel_lengths(&mut self) {
        self.channel1.common.tick_channel_length();
        self.channel2.common.tick_channel_length();
        self.channel3.common.tick_channel_length();
        self.channel4.common.tick_channel_length();
    }
}

impl Mem for Apu {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff10..=0xff14 => self.channel1.read(addr),
            0xff15..=0xff19 => self.channel2.read(addr),
            0xff1a..=0xff1e => self.channel3.read(addr),
            0xff1f..=0xff23 => self.channel4.read(addr),
            0xff24 => {
                let b7 = if self.vin_left_enable { 0x80 } else { 0 };
                let b654 = self.left_volume << 4;
                let b3 = if self.vin_right_enable { 0x08 } else { 0 };
                let b210 = self.right_volume;

                b7 | b654 | b3 | b210
            }
            0xff25 => {
                self.channel_enabled.bits
            }
            0xff26 => {
                let mut b = 0;
                b.set_bit(7, self.enabled);
                b.set_bit(0, self.channel1.common.channel_enabled);
                b.set_bit(1, self.channel2.common.channel_enabled);
                b.set_bit(2, self.channel3.common.channel_enabled);
                b.set_bit(3, self.channel4.common.channel_enabled);
                b | 0x70
            }
            0xff27..=0xff2f => 0xff,
            0xff30..=0xff3f => self.channel3.read(addr),
            _ => unreachable!("APU: Read from unmapped address: {:04X}", addr)
        }
    }

    fn write(&mut self, addr: u16, v: u8) {
        match addr {
            0xff26 => {
                let enable_apu = v.get_bit(7);

                if self.enabled && !enable_apu {
                    self.clear_all_registers()
                } else if !self.enabled && enable_apu {
                    self.frame_sequencer = 0;
                }

                println!("enable apu {}", enable_apu);
                self.enabled = enable_apu;
            }
            0xff30..=0xff3f => self.channel3.write(addr, v),

            0xff11 if !self.enabled => self.channel1.write(addr, v & 0x3f),
            0xff16 if !self.enabled => self.channel2.write(addr, v & 0x3f),
            0xff1b if !self.enabled => self.channel3.write(addr, v),
            0xff20 if !self.enabled => self.channel4.write(addr, v & 0x3f),
            0xff10..=0xff14 => self.channel1.write(addr, v),
            0xff15..=0xff19 => self.channel2.write(addr, v),
            0xff1a..=0xff1e => self.channel3.write(addr, v),
            0xff1f..=0xff23 => self.channel4.write(addr, v),

            0xff27..=0xff2f => {}

            0xff24 => {
                self.right_volume = v & 0b111;
                self.vin_right_enable = v.get_bit(3);
                self.left_volume = (v >> 4) & 0b111;
                self.vin_left_enable = v.get_bit(7);
            }
            0xff25 => {
                self.channel_enabled = ChannelEnabled::from_bits_truncate(v);
            }

            _ => unreachable!("APU: Write to unmapped address: {:04X}", addr)
        }
    }
}
