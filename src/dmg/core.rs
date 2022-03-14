use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::io::Read;
use std::rc::Rc;

use dmg::cpu::ProcessingUnit;
use dmg::mem::{MemoryBus, ROM_END, RomBuffer};

pub struct Core {
    bus: Rc<RefCell<MemoryBus>>,
    cpu: ProcessingUnit,
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

        let shared_bus: Rc<RefCell<_>> = Rc::new(RefCell::new(memory));

        Self {
            cpu: ProcessingUnit::new(shared_bus.clone()),
            bus: shared_bus,
        }
    }

    pub fn step(&mut self, buffer: &mut Vec<u32>) -> bool {
        let elapsed = self.cpu.next();

        self.bus.borrow_mut().ppu.next(elapsed, buffer)
    }

    pub fn read_rom_name(&self) -> String {
        let mut title = String::new();
        for i in 0x134..0x143 {
            let i1 = self.bus.borrow().read_byte(i);
            if i1 == 0 {
                break;
            }

            title += &(i1 as char).to_string();
        }

        title
    }
}
