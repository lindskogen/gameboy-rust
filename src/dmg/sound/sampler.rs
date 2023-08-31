use serde::{Deserialize, Serialize};
use crate::emulator::audio::AudioPlayer;

use super::{Apu, ChannelEnabled};

pub type StereoSample = (f32, f32);

#[derive(Serialize, Deserialize)]
pub struct AudioSampler {
    clock: u32,
}

impl Default for AudioSampler {
    fn default() -> Self {
        Self { clock: 0 }
    }
}

impl AudioSampler {
    pub fn tick(&mut self, apu: &Apu, audio_player: &mut AudioPlayer)  {
        self.clock += 1;

        if self.clock > 95 {
            self.clock -= 95;
            let mut audio_buffer = audio_player.buffer.lock().unwrap();
            audio_buffer.push(apu.sample());
        }
    }
}

impl Apu {
    pub fn sample(&self) -> StereoSample {
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
