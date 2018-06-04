// 32K memory
// 0x0000 - 0x7FFF

// 0x0000 - 0x00FF: Destination address for RST instructions and starting address for interrupts
// 0x0100 - 0x014F: ROM area for storing data such as the name of the game
//          0x0150: Starting address of the user program

// 0x8000 - 0x9FFF: RAM for LCD display
//                  Only 8KB is used for DMG

use std::fmt;
use std::ops::Index;

const MEM_SIZE: usize = 32_000;

pub struct Memory {
    memory: [u8; MEM_SIZE],
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            memory: [0x06; MEM_SIZE]
        }
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Memory 90-110: {:?}", &self.memory[90..110])
    }
}

impl Index<u16> for Memory {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.memory[index as usize]
    }
}


