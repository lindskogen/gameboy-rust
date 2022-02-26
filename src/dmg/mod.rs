use std::fs::File;
use std::io;
use std::io::Read;

use self::cpu::ProcessingUnit;
use self::mem::Memory;

mod cpu;
mod mem;
mod debug;

pub struct Core {
    cpu: ProcessingUnit,
}

fn read_bootloader_file(filename: &str) -> io::Result<[u8; 256]> {
    let mut buffer = [0; 256];
    let mut f = File::open(filename)?;

    f.read(&mut buffer)?;

    Ok(buffer)
}

impl Core {
    pub fn start(filename: Option<&str>) {
        let buffer = filename.and_then(|name| read_bootloader_file(name).ok());

        let memory = buffer.map(Memory::new).unwrap_or_else(|| Memory::default());

        let mut core = Core {
            cpu: ProcessingUnit::new(memory),
        };

        // println!("{:?}", core.cpu);

        for _ in 0..10_000 {
            core.cpu.next();
            // println!("{:?}", core.cpu);
        }
    }
}
