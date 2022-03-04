// 32K memory
// 0x0000 - 0x7FFF

// 0x0000 - 0x00FF: Destination address for RST instructions and starting address for interrupts
// 0x0100 - 0x014F: ROM area for storing data such as the name of the game
//          0x0150: Starting address of the user program

// 0x8000 - 0x9FFF: RAM for LCD display
//                  Only 8KB is used for DMG

use std::fmt;
use std::ops::Index;

use dmg::gpu::{GPU, VRAM_BEGIN, VRAM_END};

const ROM_BEGIN: usize = 0x100;
const ROM_END: usize = 0x7fff;

pub const ROM_SIZE: usize = ROM_END - ROM_BEGIN + 1;

const MEM_SIZE: usize = 0xffff;

pub type RomBuffer = [u8; ROM_SIZE];

pub struct MemoryBus {
    memory: [u8; MEM_SIZE],
    rom: Option<RomBuffer>,
    gpu: GPU,
}

impl Default for MemoryBus {
    fn default() -> Self {
        MemoryBus {
            memory: [0x00; MEM_SIZE],
            rom: None,
            gpu: GPU::new(),
        }
    }
}

impl MemoryBus {
    pub fn new(bootloader: [u8; 256], rom: Option<RomBuffer>) -> MemoryBus {
        let mut memory = [0x00; MEM_SIZE];

        memory[..256].copy_from_slice(&bootloader);

        MemoryBus { memory, rom, gpu: GPU::new() }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        let address = addr as usize;

        match address {
            VRAM_BEGIN..=VRAM_END => self.gpu.write_vram(address - VRAM_BEGIN, value),
            _ => self.memory[address] = value
        }
    }

    pub fn copy_vram_into_buffer(&self, buffer: &mut Vec<u32>) {
        self.gpu.copy_vram_into_buffer(buffer);
    }
}

impl fmt::Debug for MemoryBus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bootloader: {:02x?}", &self.memory[..256])
    }
}

impl Index<u16> for MemoryBus {
    type Output = u8;

    fn index(&self, addr: u16) -> &Self::Output {
        let address = addr as usize;

        match address {
            VRAM_BEGIN..=VRAM_END => self.gpu.read_vram(address - VRAM_BEGIN),
            ROM_BEGIN..=ROM_END => {
                if let Some(ref rom) = self.rom {
                    &rom[address]
                } else {
                    &self.memory[address]
                }
            }
            _ => &self.memory[address]
        }
    }
}
