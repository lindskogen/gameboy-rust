use crate::dmg::sound::common::{ChannelCommon, DUTY_TABLE};
use crate::dmg::sound::traits::{Mem, Tick};
use crate::dmg::sound::volume_envelope::VolumeEnvelope;

pub struct Channel3 {
    pub common: ChannelCommon,

}


impl Default for Channel3 {
    fn default() -> Self {
        Self {
            common: ChannelCommon::new(4),
        }
    }
}

impl Mem for Channel3 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            _ => 0xff
        }
    }

    fn write(&mut self, addr: u16, v: u8) {}
}


impl Tick for Channel3 {
    fn tick(&mut self) {
        // TODO implement!
    }
}
