// 32K memory
// 0x0000 - 0x7FFF

// 0x0000 - 0x00FF: Destination address for RST instructions and starting address for interrupts
// 0x0100 - 0x014F: ROM area for storing data such as the name of the game
//          0x0150: Starting address of the user program

// 0x8000 - 0x9FFF: RAM for LCD display
//                  Only 8KB is used for DMG

use std::fmt;

use dmg::gpu::{GPU, VRAM_BEGIN, VRAM_END};

const ROM_BEGIN: usize = 0x100;
pub const ROM_END: usize = 0x7fff;


const MEM_SIZE: usize = 0xffff + 1;

pub type RomBuffer = [u8; ROM_END];

pub struct MemoryBus {
    memory: [u8; MEM_SIZE],
    rom: Option<RomBuffer>,
    cartridge_loaded: bool,
    pub gpu: GPU,
}

impl Default for MemoryBus {
    fn default() -> Self {
        MemoryBus {
            memory: [0x00; MEM_SIZE],
            rom: None,
            cartridge_loaded: false,
            gpu: GPU::new(),
        }
    }
}

impl MemoryBus {
    pub fn new(bootloader: [u8; 256], rom: Option<RomBuffer>) -> MemoryBus {
        let mut memory = [0x00; MEM_SIZE];

        memory[..256].copy_from_slice(&bootloader);

        MemoryBus { memory, rom, gpu: GPU::new(), cartridge_loaded: false }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        let address = addr as usize;

        match address {
            0xff50 => { self.cartridge_loaded = value == 1; }
            0xff07 => {
                // println!("Timer Control: {:b}", value);
                self.memory[address] = value
            }
            VRAM_BEGIN..=VRAM_END => self.gpu.write_vram(addr, value),
            0xffff => self.gpu.write_vram(addr, value),
            _ => self.memory[address] = value
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        let address = addr as usize;

        match address {
            VRAM_BEGIN..=VRAM_END => self.gpu.read_vram(addr),
            0xffff => self.gpu.read_vram(addr),
            0..=ROM_BEGIN if self.cartridge_loaded => {
                if let Some(ref rom) = self.rom {
                    rom[address]
                } else {
                    self.memory[address]
                }
            }
            ROM_BEGIN..=ROM_END => {
                if let Some(ref rom) = self.rom {
                    rom[address]
                } else {
                    self.memory[address]
                }
            }
            _ => self.memory[address]
        }
    }

    pub fn debug_vram_into_buffer(&self, buffer: &mut Vec<u32>) {
        self.gpu.debug_vram_into_buffer(buffer);
    }
}

impl fmt::Debug for MemoryBus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bootloader: {:02x?}", &self.memory[..256])
    }
}
