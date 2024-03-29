use std::fs::File;
use std::io;
use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::dmg::cpu::ProcessingUnit;
use crate::dmg::input::JoypadInput;
use crate::dmg::mem::{MemoryBus, RomBuffer};
use crate::dmg::sound::sampler::{AudioSampler, StereoSample};
use crate::dmg::traits::Tick;
use crate::emulator::audio::AudioPlayer;

#[derive(Serialize, Deserialize)]
pub struct Core {
    bus: MemoryBus,
    cpu: ProcessingUnit,
    audio_sampler: AudioSampler,
}

fn read_rom_file(filename: &str) -> io::Result<RomBuffer> {
    let mut buffer = vec![];
    let mut f = File::open(filename)?;

    f.read_to_end(&mut buffer)?;

    Ok(buffer)
}

fn read_bootloader_file(filename: &str) -> io::Result<[u8; 256]> {
    let mut buffer = [0; 256];
    let mut f = File::open(filename)?;

    f.read(&mut buffer)?;

    Ok(buffer)
}

impl Core {
    pub fn load(boot_rom: &str, game_rom: Option<String>) -> Core {
        let boot_rom_buffer = read_bootloader_file(boot_rom).expect("Failed to read boot rom");
        let game_rom_buffer =
            game_rom.map(|filename| read_rom_file(&filename).expect("Failed to read game rom"));

        Self {
            cpu: ProcessingUnit::new(),
            bus: MemoryBus::new(Some(boot_rom_buffer), game_rom_buffer),
            audio_sampler: AudioSampler::default()
        }
    }

    pub fn load_without_boot_rom(game_rom: Option<String>) -> Core {
        let game_rom_buffer =
            game_rom.map(|filename| read_rom_file(&filename).expect("Failed to read game rom"));

        let mut cpu = ProcessingUnit::new();
        cpu.skip_boot_rom();

        Self {
            cpu,
            bus: MemoryBus::new_without_boot_rom(game_rom_buffer),
            audio_sampler: AudioSampler::default()
        }
    }

    pub fn initialize_gameboy_doctor(&mut self) {
        self.cpu.initialize_gameboy_doctor();
        self.bus.ppu.initialize_gameboy_doctor();
    }

    pub fn step(&mut self, buffer: &mut Vec<u32>, audio_player: &mut AudioPlayer, keys_pressed: JoypadInput) -> bool {
        self.bus.input.update(keys_pressed);
        let elapsed = self.cpu.next(&mut self.bus);

        let should_render = self.bus.ppu.next(elapsed, buffer);

        for _ in 0..elapsed {
            self.bus.apu.tick();
            self.audio_sampler.tick(&self.bus.apu, audio_player);
        }

        should_render
    }

    pub fn read_rom_name(&self) -> String {
        let mut title = String::new();
        for i in 0x134..0x143 {
            let i1 = self.bus.read_byte(i);
            if i1 == 0 {
                break;
            }

            title += &(i1 as char).to_string();
        }

        title
    }
}
