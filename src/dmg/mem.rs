// 32K memory
// 0x0000 - 0x7FFF

// 0x0000 - 0x00FF: Destination address for RST instructions and starting address for interrupts
// 0x0100 - 0x014F: ROM area for storing data such as the name of the game
//          0x0150: Starting address of the user program

// 0x8000 - 0x9FFF: RAM for LCD display
//                  Only 8KB is used for DMG

use hex_slice::AsHex;
use std::fmt;
use std::ops::Index;

const MEM_SIZE: usize = 32_000;

pub struct Memory {
    memory: [u8; MEM_SIZE],
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            memory: [0x00; MEM_SIZE]
        }
    }
}

impl Memory {
    pub fn new(bootloader: [u8; 256]) -> Memory {
        let mut memory = [0; MEM_SIZE];

        memory[..256].copy_from_slice(&bootloader);

        Memory {
            memory
        }
    }

    pub fn set_at(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value;
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bootloader: {:x}", &self.memory.as_hex())
    }
}

impl Index<u16> for Memory {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.memory[index as usize]
    }
}


