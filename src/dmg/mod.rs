use std::fs::File;
use std::io;
use std::io::Read;
use dmg::mem::{ROM_END, ROM_SIZE, RomBuffer};

use self::cpu::ProcessingUnit;
use self::mem::MemoryBus;

mod cpu;
mod gpu;
mod mem;
mod debug;

pub struct Core {
    pub cpu: ProcessingUnit,
}


fn read_rom_file(filename: &str) -> io::Result<RomBuffer> {
    let mut buffer = [0; ROM_END];
    let mut f = File::open(filename)?;

    f.read(&mut buffer)?;

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
        let game_rom_buffer = game_rom.map(|filename| read_rom_file(&filename).expect("Failed to read game rom"));

        let memory = MemoryBus::new(boot_rom_buffer, game_rom_buffer);

        Core {
            cpu: ProcessingUnit::new(memory)
        }
    }
}
