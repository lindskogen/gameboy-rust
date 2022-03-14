// 32K memory
// 0x0000 - 0x7FFF

// 0x0000 - 0x00FF: Destination address for RST instructions and starting address for interrupts
// 0x0100 - 0x014F: ROM area for storing data such as the name of the game
//          0x0150: Starting address of the user program

// 0x8000 - 0x9FFF: RAM for LCD display
//                  Only 8KB is used for DMG

use std::fmt;

use dmg::gpu::{GPU, VRAM_BEGIN, VRAM_END};
use std::convert::{TryFrom, TryInto};

const ROM_BEGIN: usize = 0x100;
pub const ROM_END: usize = 0x7fff;


const MEM_SIZE: usize = 0xffff + 1;

pub type RomBuffer = [u8; ROM_END];

pub struct MemoryBus {
    memory: [u8; MEM_SIZE],
    rom: Option<RomBuffer>,
    boot_rom_disabled: bool,
    mbc: MBC,
    pub ppu: GPU,
}

impl Default for MemoryBus {
    fn default() -> Self {
        MemoryBus {
            memory: [0x00; MEM_SIZE],
            rom: None,
            mbc: MBC::default(),
            ppu: GPU::new(),
            boot_rom_disabled: false,
        }
    }
}

impl MemoryBus {
    pub fn new(bootloader: [u8; 256], rom: Option<RomBuffer>) -> MemoryBus {
        let mut memory = [0x00; MEM_SIZE];

        memory[..256].copy_from_slice(&bootloader);

        let mbc = rom.and_then(|rom| rom[0x147].try_into().ok()).unwrap_or_default();

        match mbc {
            MBC::NoMBC => {}
            _ => panic!("No support for cartridge type: {:?}", mbc)
        }

        MemoryBus { memory, rom, mbc, ppu: GPU::new(), boot_rom_disabled: false }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        let address = addr as usize;

        match address {
            0xff50 => { self.boot_rom_disabled = value == 1; }
            0xff07 => {
                // println!("Timer Control: {:b}", value);
                self.memory[address] = value
            }
            VRAM_BEGIN..=VRAM_END => self.ppu.write_vram(addr, value),
            0xffff => self.ppu.write_vram(addr, value),
            _ => self.memory[address] = value
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        let address = addr as usize;

        match address {
            VRAM_BEGIN..=VRAM_END => self.ppu.read_vram(addr),
            0xffff => self.ppu.read_vram(addr),
            0..=ROM_BEGIN if self.boot_rom_disabled => self.read_rom_with_ram_fallback(address),
            ROM_BEGIN..=ROM_END => self.read_rom_with_ram_fallback(address),
            _ => self.memory[address]
        }
    }

    fn read_rom_with_ram_fallback(&self, address: usize) -> u8 {
        match self.rom {
            Some(ref rom) => {
                rom[address]
            }
            None => {
                self.memory[address]
            }
        }
    }

    pub fn debug_vram_into_buffer(&self, buffer: &mut Vec<u32>) {
        self.ppu.debug_vram_into_buffer(buffer);
    }

    pub fn get_cartridge_type(&self) -> MBC {
        self.mbc
    }
}

impl fmt::Debug for MemoryBus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bootloader: {:02x?}", &self.memory[..256])
    }
}


#[derive(Debug, Copy, Clone)]
pub enum MBC {
    NoMBC = 0x00,
    MBC1 = 0x01,
    Mbc1ExternalRam = 0x02,
    Mbc1BatteryExternalRam = 0x03,
}

impl Default for MBC {
    fn default() -> Self {
        MBC::NoMBC
    }
}

impl TryFrom<u8> for MBC {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(MBC::NoMBC),
            0x01 => Ok(MBC::MBC1),
            0x02 => Ok(MBC::Mbc1ExternalRam),
            0x03 => Ok(MBC::Mbc1BatteryExternalRam),
            _ => Err(())
        }
    }
}
