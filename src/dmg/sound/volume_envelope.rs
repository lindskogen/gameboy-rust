use crate::dmg::sound::traits::Tick;

pub struct VolumeEnvelope {
    timer: u8,
    period: u8,
    clock: u8,
    add_mode: bool,
    starting_volume: u8,
    volume: u8,
    finished: bool,
}

impl VolumeEnvelope {
    pub fn get_volume(&self) -> u8 {
        if self.period > 0 {
            self.volume
        } else {
            self.starting_volume
        }
    }
}

impl Default for VolumeEnvelope {
    fn default() -> Self {
        Self {
            timer: 0,
            period: 0,
            starting_volume: 0,
            volume: 0,
            finished: true,
            add_mode: false,
            clock: 0,
        }
    }
}


impl Tick for VolumeEnvelope {
    fn tick(&mut self) {
        if self.finished {
            return;
        }

        self.timer -= 1;

        if self.timer <= 0 {
            self.timer = if self.period != 0 { self.period } else { 8 };

            if self.add_mode && self.volume < 15 {
                self.volume += 1;
            }
            if !self.add_mode && self.volume > 0 {
                self.volume -= 1;
            }
            if self.volume == 0 || self.volume == 15 {
                self.finished = true;
            }
        }
    }
}
