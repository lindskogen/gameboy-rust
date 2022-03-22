// 32K memory
// 0x0000 - 0x7FFF

// 0x0000 - 0x00FF: Destination address for RST instructions and starting address for interrupts
// 0x0100 - 0x014F: ROM area for storing data such as the name of the game
//          0x0150: Starting address of the user program

// 0x8000 - 0x9FFF: RAM for LCD display
//                  Only 8KB is used for DMG

use std::fmt;

use crate::dmg::gpu::{GPU, VRAM_BEGIN, VRAM_END};
use crate::dmg::intf::InterruptFlag;

const ROM_BEGIN: usize = 0x100;
pub const ROM_END: usize = 0x7fff;


const ERAM_BEGIN: usize = 0xa000;
const ERAM_END: usize = 0xbfff;

const ERAM_SIZE: usize = ERAM_END - ERAM_BEGIN + 1;


const MEM_SIZE: usize = 0xffff + 1;

pub type RomBuffer = Vec<u8>;

pub struct MemoryBus {
    memory: [u8; MEM_SIZE],
    external_memory: [u8; ERAM_SIZE],
    rom: Option<RomBuffer>,
    boot_rom_disabled: bool,
    mbc_mode: MBC,
    mbc1_params: MBC1Params,
    rom_offset: usize,
    ram_offset: usize,
    pub ppu: GPU,
    pub interrupts_enabled: InterruptFlag,
}

impl Default for MemoryBus {
    fn default() -> Self {
        MemoryBus {
            memory: [0x00; MEM_SIZE],
            external_memory: [0x00; ERAM_SIZE],
            rom: None,
            mbc_mode: MBC::default(),
            mbc1_params: MBC1Params::default(),
            ppu: GPU::new(),
            boot_rom_disabled: false,
            rom_offset: 0,
            ram_offset: 0,
            interrupts_enabled: InterruptFlag::empty(),
        }
    }
}

impl MemoryBus {
    pub fn new(bootloader: [u8; 256], rom: Option<RomBuffer>) -> MemoryBus {
        let mut memory = [0x00; MEM_SIZE];

        memory[..256].copy_from_slice(&bootloader);

        let mbc = rom.as_ref().and_then(|rom| rom[0x147].try_into().ok()).unwrap_or_default();

        match mbc {
            MBC::NoMBC | MBC::MBC1 => {}
            _ => panic!("No support for cartridge type: {:?}", mbc)
        }

        let ram_size = rom.as_ref().map(|rom| rom[0x0149]).unwrap_or(0u8);

        if ram_size > 0 {
            println!("RAM size: {}", ram_size);
        }

        MemoryBus {
            memory,
            rom,
            external_memory: [0x00; ERAM_SIZE],
            mbc_mode: mbc,
            mbc1_params: MBC1Params::default(),
            rom_offset: 0,
            ram_offset: 0,
            ppu: GPU::new(),
            interrupts_enabled: InterruptFlag::empty(),
            boot_rom_disabled: false,
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        let address = addr as usize;

        match address {
            0x0000..=0x1fff => {
                match self.mbc_mode {
                    MBC::NoMBC => {}
                    MBC::MBC1 | MBC::Mbc1ExternalRam => {
                        self.mbc1_params.ram_on = (value & 0x0F) == 0x0A;
                    }
                    _ => {}
                }
            }
            0x2000..=0x3fff => {
                match self.mbc_mode {
                    MBC::NoMBC | MBC::MBC1 | MBC::Mbc1ExternalRam => {
                        let value = value & 0x1f;
                        let value = if value == 0 { 1 } else { value };
                        self.mbc1_params.rom_bank = (self.mbc1_params.rom_bank & 0x60) + value as u16;
                        self.rom_offset = self.mbc1_params.rom_bank as usize * 0x4000;
                    }
                    _ => {}
                }
            }
            0x4000..=0x5fff => {
                match self.mbc1_params.mode {
                    MBC1Mode::RamMode => {
                        self.mbc1_params.ram_bank = value as u16 & 3;
                        self.ram_offset = self.mbc1_params.ram_bank as usize * 0x2000;
                    }
                    MBC1Mode::RomMode => {
                        self.mbc1_params.rom_bank = (self.mbc1_params.rom_bank & 0x1f) + ((value as u16 & 3) << 5);
                        self.rom_offset = self.mbc1_params.rom_bank as usize * 0x4000;
                    }
                }
            }
            0x6000..=0x7fff => {
                match self.mbc_mode {
                    MBC::MBC1 | MBC::Mbc1ExternalRam => {
                        self.mbc1_params.mode = if value & 1 == 1 { MBC1Mode::RamMode } else { MBC1Mode::RomMode };
                    }
                    _ => {}
                }
            }
            ERAM_BEGIN..=ERAM_END => {
                self.external_memory[self.ram_offset + (address & 0x1fff)] = value;
            }
            0xff07 => {
                // println!("Timer Control: {:b}", value);
                self.memory[address] = value
            }
            VRAM_BEGIN..=VRAM_END => self.ppu.write_vram(addr, value),
            0xff50 => { self.boot_rom_disabled = value == 1; }
            0xffff => self.interrupts_enabled = InterruptFlag::from_bits_truncate(value),
            _ => self.memory[address] = value
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        let address = addr as usize;

        match address {
            VRAM_BEGIN..=VRAM_END => self.ppu.read_vram(addr),
            0xffff => self.interrupts_enabled.bits(),
            0..=ROM_BEGIN if self.boot_rom_disabled => self.read_rom_with_ram_fallback(address),
            ROM_BEGIN..=ROM_END => self.read_rom_with_ram_fallback(address),
            _ => self.memory[address]
        }
    }

    fn read_rom_with_ram_fallback(&self, address: usize) -> u8 {
        match self.rom {
            Some(ref rom) => {
                self.read_rom(rom, address)
            }
            None => {
                self.memory[address]
            }
        }
    }
    fn read_rom(&self, rom: &RomBuffer, address: usize) -> u8 {
        match address {
            0x4000..=0x7fff => {
                rom[self.rom_offset + (address & 0x3fff)]
            }
            ERAM_BEGIN..=ERAM_END => {
                self.external_memory[self.ram_offset + (address & 0x1fff)]
            }
            _ => rom[address]
        }
    }

    pub fn debug_vram_into_buffer(&self, buffer: &mut Vec<u32>) {
        self.ppu.debug_vram_into_buffer(buffer);
    }
}

impl fmt::Debug for MemoryBus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bootloader: {:02x?}", &self.memory[..256])
    }
}

enum MBC1Mode {
    RomMode,
    RamMode,
}

struct MBC1Params {
    // Selected ROM bank
    rom_bank: u16,

    // Selected RAM bank
    ram_bank: u16,

    // RAM enable switch
    ram_on: bool,

    // ROM/RAM expansion mode
    mode: MBC1Mode,
}

impl Default for MBC1Params {
    fn default() -> Self {
        Self {
            ram_on: false,
            rom_bank: 0x0000,
            ram_bank: 0x0000,
            mode: MBC1Mode::RomMode,
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum MBC {
    NoMBC,
    MBC1,
    Mbc1ExternalRam,
    Mbc1BatteryExternalRam,
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
