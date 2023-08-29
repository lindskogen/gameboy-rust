use crate::dmg::sound::common::ChannelCommon;
use crate::dmg::sound::traits::{Mem, Tick};
use crate::dmg::sound::volume_envelope::VolumeEnvelope;

pub struct Channel4 {
    pub common: ChannelCommon,


    pub volume_envelope: VolumeEnvelope,
}


impl Default for Channel4 {
    fn default() -> Self {
        Self {
            common: ChannelCommon::new(4),
            volume_envelope: VolumeEnvelope::default(),
        }
    }
}

impl Mem for Channel4 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            _ => 0xff
        }
    }

    fn write(&mut self, addr: u16, v: u8) {}
}


impl Tick for Channel4 {
    fn tick(&mut self) {
       // TODO implement!
    }
}

impl Channel4 {
    pub fn power_off(&mut self) {
        todo!()
    }
}
